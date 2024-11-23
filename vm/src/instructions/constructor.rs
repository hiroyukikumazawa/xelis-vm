use std::collections::{HashMap, VecDeque};
use xelis_environment::EnvironmentError;
use xelis_types::{EnumValueType, Value, ValueCell};

use crate::{stack::Stack, Backend, ChunkManager, Context, VMError};
use super::InstructionResult;

pub fn new_array<'a>(_: &Backend<'a>, stack: &mut Stack, manager: &mut ChunkManager<'a>, _: &mut Context<'a>) -> Result<InstructionResult, VMError> {
    let length = manager.read_u8()?;
    let mut array = VecDeque::with_capacity(length as usize);
    for _ in 0..length {
        let pop = stack.pop_stack()?;
        array.push_front(pop);
    }

    stack.push_stack(ValueCell::Array(array.into()).into())?;
    Ok(InstructionResult::Nothing)
}

pub fn new_struct<'a>(backend: &Backend<'a>, stack: &mut Stack, manager: &mut ChunkManager<'a>, _: &mut Context<'a>) -> Result<InstructionResult, VMError> {
    let id = manager.read_u16()?;
    let struct_type = backend.get_struct_with_id(id as usize)?;

    let fields_count = struct_type.fields().len();
    let mut fields = VecDeque::with_capacity(fields_count);
    for _ in 0..fields_count {
        fields.push_front(stack.pop_stack()?);
    }

    stack.push_stack(ValueCell::Struct(fields.into(), struct_type.clone()).into())?;
    Ok(InstructionResult::Nothing)
}

pub fn new_range<'a>(_: &Backend<'a>, stack: &mut Stack, _: &mut ChunkManager<'a>, _: &mut Context<'a>) -> Result<InstructionResult, VMError> {
    let end = stack.pop_stack()?.into_value();
    let start = stack.pop_stack()?.into_value();

    if !start.is_number() || !end.is_number() {
        return Err(VMError::InvalidRangeType);
    }

    let start_type = start.as_value()?.get_type()?;
    if start_type != end.as_value()?.get_type()? {
        return Err(VMError::InvalidRangeType);
    }

    let value = Value::Range(Box::new(start.into_value()?), Box::new(end.into_value()?), start_type);
    stack.push_stack_unchecked(ValueCell::Default(value).into());
    Ok(InstructionResult::Nothing)
}

pub fn new_map<'a>(_: &Backend<'a>, stack: &mut Stack, manager: &mut ChunkManager<'a>, _: &mut Context<'a>) -> Result<InstructionResult, VMError> {
    let len = manager.read_u8()?;
    let mut map = HashMap::with_capacity(len as usize);
    for _ in 0..len {
        let value = stack.pop_stack()?;
        let key = stack.pop_stack()?.into_value();
        if key.is_map() {
            return Err(EnvironmentError::InvalidKeyType.into());
        }

        map.insert(key, value);
    }

    stack.push_stack_unchecked(ValueCell::Map(map).into());
    Ok(InstructionResult::Nothing)
}

pub fn new_enum<'a>(backend: &Backend<'a>, stack: &mut Stack, manager: &mut ChunkManager<'a>, _: &mut Context<'a>) -> Result<InstructionResult, VMError> {
    let id = manager.read_u16()?;
    let enum_type = backend.get_enum_with_id(id as usize)?;

    let variant_id = manager.read_u8()?;
    let variant = enum_type.get_variant(variant_id)
        .ok_or(VMError::InvalidEnumVariant)?;

    let mut values = VecDeque::with_capacity(variant.fields().len());
    for _ in variant.fields() {
        values.push_front(stack.pop_stack()?);
    }

    stack.push_stack(ValueCell::Enum(values.into(), EnumValueType::new(enum_type.clone(), variant_id)).into())?;
    Ok(InstructionResult::Nothing)
}