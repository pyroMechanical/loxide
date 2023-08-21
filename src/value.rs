use crate::vm::InterpretError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
}

impl Value {
    pub fn is_number(&self) -> bool {
        match self {
            Self::Number(_) => true,
            _ => false
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Self::Bool(_) => true,
            _ => false
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Self::Nil => true,
            _ => false
        }
    }

    pub fn is_falsey(&self) -> bool{
        match self {
            Value::Nil => true,
            Value::Bool(value) => !value,
            _ => false
        }
    }

    pub fn as_bool(&self) -> Result<bool, InterpretError> {
        match self {
            Self::Bool(value) => Ok(*value),
            _ => Err(InterpretError::Runtime)
        }
    }

    pub fn as_number(&self) -> Result<f64, InterpretError> {
        match self {
            Self::Number(value) => Ok(*value),
            _ => Err(InterpretError::Runtime)
        }
    }
}