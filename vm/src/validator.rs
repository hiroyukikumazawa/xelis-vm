use thiserror::Error;
use xelis_environment::Environment;
use xelis_types::{EnumType, EnumVariant, StructType, Type, Value, ValueError, ValueType};
use xelis_bytecode::{Module, OpCode};

use crate::ChunkReader;

#[derive(Debug, Error)]
pub enum ValidatorError<'a> {
    #[error("too much memory usage in constants")]
    TooMuchMemoryUsage,
    #[error("too many constants")]
    TooManyConstants,
    #[error("too many chunks")]
    TooManyChunks,
    #[error("too many types")]
    TooManyTypes,

    #[error("too many structs")]
    TooManyStructs,
    #[error("too many struct fields")]
    TooManyStructFields(&'a StructType),
    #[error("recursive struct")]
    RecursiveStruct(&'a StructType),

    #[error("too many enums")]
    TooManyEnums,
    #[error("too many enums variants")]
    TooManyEnumsVariants,
    #[error("too many enums variants fields")]
    TooManyEnumsVariantsFields(&'a EnumVariant),
    #[error("recursive enum")]
    RecursiveEnum(&'a EnumType),

    #[error("incorrect fields")]
    IncorrectFields,
    #[error("incorrect variant")]
    IncorrectVariant,
    #[error("invalid op code")]
    InvalidOpCode,
    #[error("invalid opcode arguments")]
    InvalidOpCodeArguments,
    #[error("invalid range")]
    InvalidRange,
    #[error("invalid range type")]
    InvalidRangeType,
    #[error("reference not allowed")]
    ReferenceNotAllowed,
    #[error(transparent)]
    ValueError(#[from] ValueError)
}

pub struct ModuleValidator<'a> {
    module: &'a Module,
    environment: &'a Environment,
    constant_max_depth: usize,
    constant_max_memory: usize
}

impl<'a> ModuleValidator<'a> {
    pub fn new(module: &'a Module, environment: &'a Environment) -> Self {
        Self { module, environment, constant_max_depth: 16, constant_max_memory: 1024 }
    }

    // Due to the use of ValuePointer, we need to clone each value as we can't keep one reference
    fn verify_value(&self, value: &ValueType) -> Result<usize, ValidatorError<'a>> {
        let mut stack = vec![(value, 0)];
        let mut memory_usage = 0;

        while let Some((value, depth)) = stack.pop() {
            if depth > self.constant_max_depth {
                return Err(ValidatorError::TooManyConstants);
            }

            // Increase by one for the byte type of the value
            memory_usage += 1;
            if memory_usage > self.constant_max_memory {
                return Err(ValidatorError::TooMuchMemoryUsage);
            }

            match value {
                ValueType::Struct(fields, t) => {
                    if fields.len() != t.fields().len() {
                        return Err(ValidatorError::IncorrectFields);
                    }
        
                    if !self.module.structs().contains(t) && !self.environment.get_structures().contains(t) {
                        return Err(ValidatorError::TooManyStructs);
                    }

                    for field in fields {
                        stack.push((field, depth + 1));
                    }
                    memory_usage += 8;
                },
                ValueType::Enum(fields, t) => {
                    let variant = t.enum_type()
                        .variants()
                        .get(t.variant_id() as usize)
                        .ok_or(ValidatorError::IncorrectVariant)?;

                    if fields.len() != variant.fields().len() {
                        return Err(ValidatorError::IncorrectFields);
                    }

                    if !self.module.enums().contains(t.enum_type()) && !self.environment.get_enums().contains(t.enum_type()) {
                        return Err(ValidatorError::TooManyEnums);
                    }

                    for field in fields {
                        stack.push((field, depth + 1));
                    }

                    memory_usage += 10;
                },
                ValueType::Array(elements) => {
                    if elements.len() > u32::MAX as usize {
                        return Err(ValidatorError::TooManyConstants);
                    }

                    for element in elements {
                        stack.push((element, depth + 1));
                    }
                    memory_usage += 4;
                },
                ValueType::Map(map) => {
                    if map.len() > u32::MAX as usize {
                        return Err(ValidatorError::TooManyConstants);
                    }

                    for (key, value) in map {
                        stack.push((key, depth + 1));
                        stack.push((value, depth + 1));
                    }
                    memory_usage += 16;
                },

                ValueType::Optional(opt) => {
                    if let Some(value) = opt {
                        memory_usage += 1;
                        stack.push((value, depth + 1));
                    }
                },
                ValueType::Default(v) => match v {
                    Value::Range(left, right, _type) => {
                        if !left.is_number() || !right.is_number() {
                            return Err(ValidatorError::InvalidRange);
                        }
    
                        let left_type = left.get_type()?;
                        if left_type != right.get_type()? {
                            return Err(ValidatorError::InvalidRange);
                        }
    
                        if left_type != *_type {
                            return Err(ValidatorError::InvalidRangeType);
                        }

                        memory_usage += 4;
                    },
                    Value::Null => memory_usage += 1,
                    Value::Boolean(_) => memory_usage += 1,
                    Value::String(str) => memory_usage += str.len(),
                    Value::U8(_) => memory_usage += 1,
                    Value::U16(_) => memory_usage += 2,
                    Value::U32(_) => memory_usage += 4,
                    Value::U64(_) => memory_usage += 8,
                    Value::U128(_) => memory_usage += 16,
                    Value::U256(_) => memory_usage += 32,
                }
            }
        }

        Ok(memory_usage)
    }

    // Verify all the declared constants in the module
    fn verify_constants(&self) -> Result<(), ValidatorError<'a>> {
        let mut memory_usage = 0;
        for c in self.module.constants() {
            memory_usage += self.verify_value(c)?;

            if memory_usage > self.constant_max_memory {
                return Err(ValidatorError::TooManyConstants);
            }
        }

        Ok(())
    }

    // Verify all the declared chunks in the module
    // We verify that the opcodes are valid and that the count of arguments are correct
    fn verify_chunks(&self) -> Result<(), ValidatorError<'a>> {
        for chunk in self.module.chunks() {
            let mut reader = ChunkReader::new(chunk);
            while let Some(instruction) = reader.next_u8() {
                let op = OpCode::from_byte(instruction)
                    .ok_or(ValidatorError::InvalidOpCode)?;

                reader.advance(op.arguments_bytes())
                    .map_err(|_| ValidatorError::InvalidOpCodeArguments)?;
            }
        }

        Ok(())
    }

    // Verify the enums integrity
    fn verify_enums(&self) -> Result<(), ValidatorError<'a>> {
        // No need to check the ids, they are already checked by the Module
        for e in self.module.enums() {
            // Verify the variants
            if e.variants().len() > u8::MAX as usize {
                return Err(ValidatorError::TooManyEnumsVariants);
            }

            for variant in e.variants() {
                if variant.fields().len() > u8::MAX as usize {
                    return Err(ValidatorError::TooManyEnumsVariantsFields(variant));
                }

                for field in variant.fields() {
                    if let Type::Enum(inner) = field {
                        if e == inner {
                            return Err(ValidatorError::RecursiveEnum(e));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // Verify the structs integrity
    fn verify_structs(&self) -> Result<(), ValidatorError<'a>> {
        // No need to check the ids, they are already checked by the Module
        for s in self.module.structs() {
            if s.fields().len() > u8::MAX as usize {
                return Err(ValidatorError::TooManyStructFields(s));
            }

            for field in s.fields() {
                if let Type::Struct(inner) = field {
                    if s == inner {
                        return Err(ValidatorError::RecursiveStruct(s));
                    }
                }
            }
        }
        Ok(())
    }

    // Verify the module integrity and return an error if it's invalid
    pub fn verify(&self) -> Result<(), ValidatorError<'a>> {
        let max = u16::MAX as usize;

        // We support max of 65535 constants, chunks, structs and enums
        if self.module.constants().len() >= max {
            return Err(ValidatorError::TooManyConstants);
        }

        if self.module.chunks().len() >= max {
            return Err(ValidatorError::TooManyChunks);
        }

        if self.module.structs().len() >= max {
            return Err(ValidatorError::TooManyStructs);
        }

        if self.module.enums().len() >= max {
            return Err(ValidatorError::TooManyEnums);
        }

        // Maximum of 65535 types
        let max_types = self.module.structs().len() + self.module.enums().len();
        if max_types >= max {
            return Err(ValidatorError::TooManyTypes);
        }

        self.verify_enums()?;
        self.verify_structs()?;
        self.verify_constants()?;
        self.verify_chunks()?;

        Ok(())
    }
}