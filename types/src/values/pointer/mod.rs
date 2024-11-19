mod sub_value;
mod weak;
mod inner;

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr,
};

use super::Value;

pub use sub_value::SubValue;
pub use weak::WeakValue;
pub use inner::ValuePointerInner;

// Value Pointer is a wrapper around the real Value Pointer
// It was introduced to allow to implement a custom Drop to prevent any stackoverflow
// that could happen with huge nested values
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValuePointer(ValuePointerInner);

impl ValuePointer {
    pub fn from(value: SubValue) -> Self {
        Self(ValuePointerInner::Shared(value))
    }

    #[inline(always)]
    pub fn owned(value: Value) -> Self {
        Self(ValuePointerInner::Owned(Box::new(value)))
    }

    #[inline(always)]
    pub fn shared(value: SubValue) -> Self {
        Self(ValuePointerInner::Shared(value))
    }

    #[inline(always)]
    pub fn get_value_ptr(&self) -> *const Value {
        ptr::from_ref(self.handle().as_value())
    }

    #[inline(always)]
    pub fn set_null(&mut self) {
        match &mut self.0 {
            ValuePointerInner::Owned(v) => {
                **v = Value::Null;
            },
            ValuePointerInner::Shared(v) => {
                *v.borrow_mut() = Value::Null;
            }
        }
    }

    pub fn weak(&mut self) -> WeakValue {
        match &mut self.0 {
            ValuePointerInner::Owned(v) => {
                let dst = std::mem::take(v);
                let shared = SubValue::new(*dst);
                let weak = shared.downgrade();
                self.0 = ValuePointerInner::Shared(shared);

                weak
            },
            ValuePointerInner::Shared(v) => v.downgrade()
        }
    }

    pub fn shareable(&mut self) -> Self {
        match &mut self.0 {
            ValuePointerInner::Owned(v) => {
                let dst = std::mem::take(v);
                let shared = SubValue::new(*dst);
                self.0 = ValuePointerInner::Shared(shared.clone());
                ValuePointer::shared(shared)
            },
            ValuePointerInner::Shared(v) => ValuePointer::shared(v.clone())
        }
    }

    #[inline(always)]
    pub fn into_inner(&mut self) -> Value {
        let v = std::mem::take(&mut self.0);
        v.into_inner()
    }

    #[inline(always)]
    pub fn into_ownable(&mut self) -> ValuePointer {
        let v = std::mem::take(&mut self.0);
        v.into_ownable()
    }

    pub fn to_value(&self) -> Value {
        self.handle().as_value().clone()
    }
}

impl AsRef<ValuePointerInner> for ValuePointer {
    fn as_ref(&self) -> &ValuePointerInner {
        &self.0
    }
}

impl AsMut<ValuePointerInner> for ValuePointer {
    fn as_mut(&mut self) -> &mut ValuePointerInner {
        &mut self.0
    }
}

impl Deref for ValuePointer {
    type Target = ValuePointerInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ValuePointer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Hash for ValuePointer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.handle()
            .hash_with_tracked_pointers(state, &mut HashSet::new());
    }
}

impl Drop for ValuePointer {
    fn drop(&mut self) {
        let mut stack = vec![self.into_inner()];

        while let Some(value) = stack.pop() {
            match value {
                Value::Map(map) => {
                    for (k, mut v) in map {
                        stack.push(k);
                        stack.push(v.into_inner());
                    }
                },
                Value::Array(array) => {
                    stack.extend(array.into_iter().map(|mut v| v.into_inner()));
                },
                Value::Optional(Some(mut v)) => {
                    stack.push(v.into_inner());
                },
                Value::Struct(fields, _) => {
                    stack.extend(fields.into_iter().map(|mut v| v.into_inner()));
                },
                Value::Enum(fields, _) => {
                    stack.extend(fields.into_iter().map(|mut v| v.into_inner()));
                }
                _ => {}
            }
        }
    }
}