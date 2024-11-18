mod inner;
mod weak;

use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    ptr,
    rc::Rc
};

use crate::{ValueHandle, ValueHandleMut};
use super::Value;

pub use inner::InnerValue;
pub use weak::WeakValue;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum ValuePointerInner {
    Owned(Box<Value>),
    Shared(InnerValue)
}

impl Default for ValuePointerInner {
    fn default() -> Self {
        Self::Owned(Box::new(Value::Null))
    }
}

// Value Pointer is a wrapper around the real Value Pointer
// It was introduced to allow to implement a custom Drop to prevent any stackoverflow
// that could happen with huge nested values
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValuePointer(ValuePointerInner);

impl ValuePointer {
    pub fn from(value: InnerValue) -> Self {
        Self(ValuePointerInner::Shared(value))
    }

    #[inline(always)]
    pub fn owned(value: Value) -> Self {
        Self(ValuePointerInner::Owned(Box::new(value)))
    }

    #[inline(always)]
    pub fn shared(value: InnerValue) -> Self {
        Self(ValuePointerInner::Shared(value))
    }

    #[inline(always)]
    pub fn get_value_ptr(&self) -> *const Value {
        ptr::from_ref(self.handle().as_value())
    }

    pub fn weak(&mut self) -> WeakValue {
        match &mut self.0 {
            ValuePointerInner::Owned(v) => {
                let dst = std::mem::take(v);
                let shared = InnerValue::new(*dst);
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
                let shared = InnerValue::new(*dst);
                self.0 = ValuePointerInner::Shared(shared.clone());
                ValuePointer::shared(shared)
            },
            ValuePointerInner::Shared(v) => ValuePointer::shared(v.clone())
        }
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

impl ValuePointer {
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
}

impl Hash for ValuePointer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.handle()
            .hash_with_tracked_pointers(state, &mut HashSet::new());
    }
}

impl Drop for ValuePointer {
    fn drop(&mut self) {
        let self_pointer = self.get_value_ptr();

        let mut stack = vec![self.into_inner()];
        let new_pointer = self.get_value_ptr();
        assert_ne!(self_pointer, new_pointer);

        // We need to prevent any stackoverflow that could happen due to the recursive reference cyles
        let mut visited = HashSet::new();

        while let Some(value) = stack.pop() {
            match value {
                Value::Map(map) => {
                    for (k, mut v) in map {
                        stack.push(k);

                        if visited.insert(v.get_value_ptr()) {
                            stack.push(v.into_inner());
                        }
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

impl ValuePointerInner {
    // Convert into a owned value
    // Clone if the value is shared and can't be moved
    pub fn into_inner(self) -> Value {
        match self {
            Self::Owned(v) => *v,
            Self::Shared(v) => match Rc::try_unwrap(v.into_inner()) {
                Ok(value) => value.into_inner(),
                Err(rc) => rc.borrow().clone()
            }
        }
    }

    // Clone the Rc value to fully own it
    pub fn into_ownable(self) -> ValuePointer {
        ValuePointer(match self {
            Self::Owned(_) => self,
            Self::Shared(v) => Self::Owned(Box::new(match Rc::try_unwrap(v.into_inner()) {
                Ok(value) => value.into_inner(),
                Err(rc) => rc.borrow().clone()
            }))
        })
    }

    // Transform the value into a shared value
    pub fn transform(&mut self) -> ValuePointer {
        ValuePointer(match self {
            Self::Owned(v) => {
                let dst = std::mem::replace(v, Box::new(Value::Null));
                let shared = Self::Shared(InnerValue::new(*dst));
                *self = shared.clone();
                shared
            },
            Self::Shared(v) => Self::Shared(v.clone())
        })
    }

    // Wrap the value into an handle to be casted to a reference of the value
    pub fn handle<'a>(&'a self) -> ValueHandle<'a> {
        match self {
            Self::Owned(v) => ValueHandle::Borrowed(v),
            Self::Shared(v) => ValueHandle::Ref(v.borrow())
        }
    }

    // Wrap the value into an handle to be casted to a mutable reference of the value
    pub fn handle_mut<'a>(&'a mut self) -> ValueHandleMut<'a> {
        match self {
            Self::Owned(v) => ValueHandleMut::Borrowed(v),
            Self::Shared(v) => ValueHandleMut::RefMut(v.borrow_mut())
        }
    }
}