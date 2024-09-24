use crate::{Path, State, Type, Value};

use super::EnvironmentError;

// first parameter is the current value / instance
// second is the list of all parameters for this function call
pub type FnReturnType = Result<Option<Value>, EnvironmentError>;
pub type FnInstance<'a> = Result<&'a mut Value, EnvironmentError>;
pub type FnParams<'a> = Vec<Path<'a>>;
pub type OnCallFn = fn(FnInstance, FnParams) -> FnReturnType;

// Native function that is implemented in Rust
// This is used to register functions in the environment
#[derive(Debug)]
pub struct NativeFunction {
    // function on type
    for_type: Option<Type>,
    parameters: Vec<Type>,
    on_call: OnCallFn,
    // cost for each call
    cost: u64,
    // expected type of the returned value
    return_type: Option<Type>
}

impl NativeFunction {
    // Create a new instance of the NativeFunction
    pub fn new(for_type: Option<Type>, parameters: Vec<Type>, on_call: OnCallFn, cost: u64, return_type: Option<Type>) -> Self {
        Self {
            for_type,
            parameters,
            on_call,
            cost,
            return_type
        }
    }

    // Execute the function
    pub fn call_function(&self, instance_value: Option<&mut Value>, parameters: FnParams, state: &mut State) -> Result<Option<Value>, EnvironmentError> {
        if parameters.len() != self.parameters.len() || (instance_value.is_some() != self.for_type.is_some()) {
            return Err(EnvironmentError::InvalidFnCall)
        }

        // TODO
        state.increase_gas_usage(self.cost).unwrap();

        let instance = match instance_value {
            Some(v) => Ok(v),
            None => Err(EnvironmentError::FnExpectedInstance)
        };
        (self.on_call)(instance, parameters)
    }

    // Get parameters of the function
    pub fn get_parameters(&self) -> &Vec<Type> {
        &self.parameters
    }

    // Get the expected type of the returned value
    pub fn return_type(&self) -> &Option<Type> {
        &self.return_type
    }
}