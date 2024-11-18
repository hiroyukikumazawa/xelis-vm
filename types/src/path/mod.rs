mod handle;

use crate::{values::{Value, ValueError}, ValuePointer, WeakValue};
pub use handle::{
    ValueHandle,
    ValueHandleMut
};

#[macro_export]
macro_rules! path_as_ref {
    ($path: expr, $name: ident, $body: block) => {
        match $path {
            Path::Owned(ref $name) => $body,
            Path::Borrowed(ref $name) => $body,
            Path::Wrapper(ref v) => {
                let pointer = v.upgrade().ok_or(ValueError::WeakValue)?;
                let handle = pointer.handle();
                let $name = handle.as_ref();
                $body
            }
        };
    };
    ($path: expr, $name: ident, $expr: expr) => {
        path_as_ref!($path, $name, { $expr })
    };
    ($path: expr, $expr: expr) => {
        path_as_ref!($path, value, { $expr })
    };
}

#[macro_export]
macro_rules! path_as_mut {
    ($path: expr, $name: ident, $body: block) => {
        match $path {
            Path::Owned($name) => {
                $body
            },
            Path::Borrowed(v) => {
                let mut cloned = v.clone();
                let $name = &mut cloned;
                let res = $body;
                *$path = Path::Owned(cloned);
                res
            },
            Path::Wrapper(v) => {
                let mut pointer = v.upgrade().ok_or(ValueError::WeakValue)?;
                let mut handle = pointer.handle_mut();
                let $name = handle.as_value_mut();
                $body
            }
        }
    };
    ($path: expr, $name: ident, $expr: expr) => {
        path_as_mut!($path, $name, { $expr })
    };
    ($path: expr, $expr: expr) => {
        path_as_mut!($path, value, { $expr })
    };
}

// Temporary holder for a value
// This is used to allow to have a reference to a value
#[derive(Debug, Clone)]
pub enum Path<'a> {
    Owned(Value),
    // Used for constants
    Borrowed(&'a Value),
    Wrapper(WeakValue)
}

impl<'a> Path<'a> {
    // Get the sub value of the path
    pub fn get_sub_variable(self, index: usize) -> Result<Path<'a>, ValueError> {
        match self {
            Self::Owned(v) => {
                let mut values = v.to_sub_vec()?;
                let len = values.len();
                if index >= len {
                    return Err(ValueError::OutOfBounds(index, len))
                }

                let mut at_index = values.remove(index);
                Ok(Path::Wrapper(at_index.weak()))
            },
            Self::Borrowed(v) => {
                let values = v.as_sub_vec()?;
                let len = values.len();
                let at_index = values
                    .get(index)
                    .ok_or_else(|| ValueError::OutOfBounds(index, len))?;

                // TODO
                Ok(Path::Wrapper(at_index.clone().weak()))
            },
            Self::Wrapper(v) => {
                let mut pointer = v.upgrade().ok_or(ValueError::WeakValue)?;
                let mut values = pointer.handle_mut();
                let values = values.as_mut_sub_vec()?;
                let len = values.len();
                let at_index = values
                    .get_mut(index)
                    .ok_or_else(|| ValueError::OutOfBounds(index, len))?;

                Ok(Path::Wrapper(at_index.weak()))
            }
        }
    }

    #[inline(always)]
    pub fn into_owned(self) -> Value {
        match self {
            Self::Owned(v) => v,
            Self::Borrowed(v) => v.clone(),
            Self::Wrapper(v) => v.upgrade().unwrap().into_inner()
        }
    }

    #[inline(always)]
    pub fn into_pointer(self) -> ValuePointer {
        match self {
            Self::Owned(v) => ValuePointer::owned(v),
            Self::Borrowed(v) => ValuePointer::owned(v.clone()),
            Self::Wrapper(v) => v.upgrade().unwrap()
        }
    }

    // Assign a value to the path
    pub fn assign(&mut self, value: Value) -> Result<(), ValueError> {
        match self {
            Self::Owned(v) => {
                *v = value;
            },
            Self::Borrowed(_) => {
                *self = Self::Owned(value);
            },
            Self::Wrapper(weak) => {
                let mut pointer = weak.upgrade().ok_or(ValueError::WeakValue)?;
                let mut handle = pointer.handle_mut();
                *handle.as_mut() = value;
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_owned() -> Result<(), ValueError> {
        let path = Path::Owned(Value::Null);
        path_as_ref!(path, value, {
            assert!(value.is_null());
        });
        Ok(())
    }

    #[test]
    fn test_macro_mut_owned() -> Result<(), ValueError> {
        let mut path = Path::Owned(Value::Null);
        path_as_mut!(&mut path, value, {
            *value = Value::Boolean(true);
        });
        assert!(path.into_owned().as_bool()?);
        Ok(())
    }
}