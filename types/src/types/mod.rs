mod r#struct;

pub use r#struct::*;

use crate::{
    values::Value,
    ValueOwnable,
};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    hash::{BuildHasher, Hash},
};


#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Type {
    // Any Type is accepted
    Any,
    // T is a generic type, inner byte is for its position
    T(u8),

    U8,
    U16,
    U32,
    U64,
    U128,
    U256,

    String,
    Bool,
    Struct(StructType),

    Array(Box<Type>),
    Optional(Box<Type>),
    Range(Box<Type>),
    Map(Box<Type>, Box<Type>),
}

impl Type {
    // transform a byte into a primitive type
    pub fn primitive_type_from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Type::U8),
            1 => Some(Type::U16),
            2 => Some(Type::U32),
            3 => Some(Type::U64),
            4 => Some(Type::U128),
            5 => Some(Type::U256),
            6 => Some(Type::Bool),
            7 => Some(Type::String),
            _ => None
        }
    }

    // get the byte representation of the primitive type
    pub fn primitive_byte(&self) -> Option<u8> {
        match self {
            Type::U8 => Some(0),
            Type::U16 => Some(1),
            Type::U32 => Some(2),
            Type::U64 => Some(3),
            Type::U128 => Some(4),
            Type::U256 => Some(5),
            Type::Bool => Some(6),
            Type::String => Some(7),
            _ => None
        }
    }

    // check if the type has an inner type
    pub fn has_inner_type(&self) -> bool {
        match self {
            Type::Array(_) | Type::Optional(_) | Type::Range(_) | Type::Map(_, _) => true,
            _ => false
        }
    }

    // check if the type is a primitive type
    pub fn is_primitive(&self) -> bool {
        self.primitive_byte().is_some()
    }

    // Get a type from a value
    pub fn from_value(value: &Value) -> Option<Self> {
        let _type = match value {
            Value::Null => return None,
            Value::U8(_) => Type::U8,
            Value::U16(_) => Type::U16,
            Value::U32(_) => Type::U32,
            Value::U64(_) => Type::U64,
            Value::U128(_) => Type::U128,
            Value::U256(_) => Type::U256,
            Value::String(_) => Type::String,
            Value::Boolean(_) => Type::Bool,
            Value::Optional(value) => Type::Optional(Box::new(match value.as_ref()? {
                ValueOwnable::Owned(v) => Type::from_value(&v)?,
                ValueOwnable::Rc(v) => Type::from_value(&v.borrow())?,
            })),
            Value::Array(values) => Type::Array(Box::new(Type::from_value(&values.first()?.handle())?)),
            Value::Struct(_, _type) => Type::Struct(_type.clone()),
            Value::Range(_, _, _type) => Type::Range(Box::new(_type.clone())),
            Value::Map(map) => {
                let (key, value) = map.iter().next()?;
                let key = Type::from_value(&key)?;
                let value = Type::from_value(&value.handle())?;
                Type::Map(Box::new(key), Box::new(value))
            },
        };

        Some(_type)
    }

    pub fn get_inner_type(&self) -> &Type {
        match &self {
            Type::Array(ref _type) => _type,
            Type::Optional(ref _type) => _type,
            Type::Range(ref _type) => _type,
            _ => &self
        }
    }

    pub fn get_generic_type(&self, id: u8) -> Option<&Type> {
        match id {
            0 => match &self {
                Type::Map(key, _) => Some(key.as_ref()),
                Type::Array(inner) => Some(inner.as_ref()),
                Type::Optional(inner) => Some(inner.as_ref()),
                Type::Range(inner) => Some(inner.as_ref()),
                _ => None
            },
            1 => match &self {
                Type::Map(_, value) => Some(value.as_ref()),
                _ => None
            }
            _ => None
        }
    }

    pub fn allow_null(&self) -> bool {
        match self {
            Type::Optional(_) => true,
            _ => false
        }
    }

    pub fn is_generic(&self) -> bool {
        match self {
            Type::T(_) | Type::Any => true,
            _ => false
        }
    }

    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match other {
            Type::Range(inner) => match self {
                Type::Range(inner2) => inner.is_compatible_with(inner2),
                _ => false
            },
            Type::Any | Type::T(_) => true,
            Type::Array(sub_type) => match self {
                Type::Array(sub) => sub.is_compatible_with(sub_type.as_ref()),
                _ => *self == *other || self.is_compatible_with(sub_type.as_ref()),
            },
            Type::Optional(sub_type) => match self {
                Type::Optional(sub) => sub.is_compatible_with(sub_type.as_ref()),
                _ => *self == *other || self.is_compatible_with(sub_type.as_ref()),
            },
            Type::Map(k, v) => match self {
                Type::Map(k2, v2) => k.is_compatible_with(k2) && v.is_compatible_with(v2),
                _ => false
            },
            o => *o == *self || self.is_generic(),
        }
    }

    // check if the type can be casted to another type
    pub fn is_castable_to(&self, other: &Type) -> bool {
        match self {
            Type::U8 => match other {
                Type::U16 | Type::U32 | Type::U64 | Type::U128 | Type::U256 | Type::String => true,
                _ => false
            },
            Type::U16 => match other {
                Type::U8 | Type::U32 | Type::U64 | Type::U128 | Type::U256 | Type::String => true,
                _ => false
            },
            Type::U32 => match other {
                Type::U8 | Type::U16 | Type::U64 | Type::U128 | Type::U256 | Type::String => true,
                _ => false
            },
            Type::U64 => match other {
                Type::U8 | Type::U16 | Type::U32 | Type::U128 | Type::U256 | Type::String => true,
                _ => false
            },
            Type::U128 => match other {
                Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U256 | Type::String => true,
                _ => false
            },
            Type::U256 => match other {
                Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 | Type::String => true,
                _ => false
            },
            Type::Bool => match other {
                Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 | Type::U256 | Type::String => true,
                _ => false
            },
            _ => false
        }
    }

    // Check if current type can be casted to another type without loss of data
    pub fn is_castable_to_no_loss(&self, other: &Type) -> bool {
        match self {
            Type::U8 => match other {
                Type::U16 | Type::U32 | Type::U64 | Type::U128 | Type::U256 => true,
                _ => false
            },
            Type::U16 => match other {
                Type::U32 | Type::U64 | Type::U128 | Type::U256 => true,
                _ => false
            },
            Type::U32 => match other {
                Type::U64 | Type::U128 | Type::U256 => true,
                _ => false
            },
            Type::U64 => match other {
                Type::U128 | Type::U256 => true,
                _ => false
            },
            Type::U128 => match other {
                Type::U256 => true,
                _ => false
            }
            _ => false
        }
    }

    pub fn is_iterable(&self) -> bool {
        match self {
            Type::Array(_) => true,
            Type::Range(_) => true,
            _ => false
        }
    }

    pub fn is_array(&self) -> bool {
        match &self {
            Type::Array(_) => true,
            _ => false
        }
    }

    pub fn is_struct(&self) -> bool {
        match &self {
            Type::Struct(_) => true,
            _ => false
        }
    }

    pub fn is_number(&self) -> bool {
        match &self {
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::U128 | Type::U256 => true,
            _ => false
        }
    }

    pub fn is_optional(&self) -> bool {
        match &self {
            Type::Optional(_) => true,
            _ => false
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Any => write!(f, "any"),
            Type::T(id) => write!(f, "T{}", id),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::U128 => write!(f, "u128"),
            Type::U256 => write!(f, "u256"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Struct(id) => write!(f, "struct({:?})", id),
            Type::Array(_type) => write!(f, "{}[]", _type),
            Type::Optional(_type) => write!(f, "optional<{}>", _type),
            Type::Range(_type) => write!(f, "range<{}>", _type),
            Type::Map(key, value) => write!(f, "map<{}, {}>", key, value),
        }
    }
}

pub trait HasKey<K> {
    fn has(&self, key: &K) -> bool;
}

impl<K: Hash + Eq> HasKey<K> for HashSet<K> {
    fn has(&self, key: &K) -> bool {
        self.contains(key)
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> HasKey<K> for HashMap<K, V, S> {
    fn has(&self, key: &K) -> bool {
        self.contains_key(key)
    }
}