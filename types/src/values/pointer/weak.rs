use std::{
    cell::RefCell,
    rc::Weak
};
use crate::Value;

use super::{SubValue, ValuePointer};


#[derive(Debug, Clone)]
pub struct WeakValue(Weak<RefCell<Value>>);

impl WeakValue {
    #[inline(always)]
    pub fn new(weak: Weak<RefCell<Value>>) -> Self {
        WeakValue(weak)
    }

    #[inline(always)]
    pub fn upgrade<'a>(&'a self) -> Option<ValuePointer> {
        self.0.upgrade().map(|v| ValuePointer::from(SubValue::from(v)))
    }
}