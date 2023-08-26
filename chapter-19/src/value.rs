use crate::vm::InterpretError;
use crate::object::{Object, ObjectType};
use std::ptr::NonNull;
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj(NonNull<Object>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>{
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Number(num) => write!(f, "{}", num),
            Self::Obj(object) => write!(f, "{}", unsafe{object.as_ref()})
        }
    }
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

    pub fn is_string(&self) -> bool {
        match self {
            Self::Obj(obj) => match unsafe {obj.as_ref().object_type} {
                ObjectType::String(_) => true,
                _ => false
            },
            _ => false
        }
    }

    pub fn as_str(&self) -> Result<&str, InterpretError> {
        match self {
            Value::Obj(object) => match unsafe{object.as_ref().object_type} {
                ObjectType::String(str) => Ok(unsafe{str.as_ref()}),
                _ => Err(InterpretError::Runtime),
            }
            _ => Err(InterpretError::Runtime)
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

fn create_string_value(string:String, objects: &mut Option<NonNull<Object>>, error_value: InterpretError) -> Result<Value, InterpretError> {
    let box_str = string.into_boxed_str();
    let ptr_str = NonNull::new(Box::leak(box_str)).ok_or(error_value)?;
    let ptr_obj = Object::new(ObjectType::String(ptr_str), objects, error_value)?;
    Ok(Value::Obj(ptr_obj))
}

pub fn copy_string<'a>(source: &'a str, objects: &mut Option<NonNull<Object>>) -> Result<Value, InterpretError> {
    let string = source.to_string();
    create_string_value(string, objects, InterpretError::Compile)
}

pub fn concatenate_strings(mut a: String, b: &str, objects: &mut Option<NonNull<Object>>) -> Result<Value, InterpretError> {
    a.push_str(b);
    create_string_value(a, objects, InterpretError::Runtime)
}