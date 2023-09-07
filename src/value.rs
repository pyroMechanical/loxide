use crate::allocate::VMOrCompiler;
use crate::vm::InterpretError;
use crate::object::{Object, ObjectType, ObjString};
use std::collections::HashSet;
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Obj(*mut Object),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>{
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Number(num) => write!(f, "{}", num),
            Self::Obj(object) =>  write!(f, "{}", Object::to_string(*object))
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

    pub fn _is_bool(&self) -> bool {
        match self {
            Self::Bool(_) => true,
            _ => false
        }
    }

    pub fn _is_nil(&self) -> bool {
        match self {
            Self::Nil => true,
            _ => false
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Self::Obj(obj) => {
                if obj.is_null() {return false;}
                match unsafe {(**obj).object_type} {
                    ObjectType::String => true,
                    _ => false
                }
            },
            _ => false
        }
    }

    pub fn as_str(&self) -> Result<&str, InterpretError> {
        match self {
            Value::Obj(obj) => match unsafe{(**obj).object_type} {
                ObjectType::String => {
                    Ok(unsafe{(*(*obj as *mut ObjString)).as_str()})
                },
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

    pub fn _as_bool(&self) -> Result<bool, InterpretError> {
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

fn create_string_value<'a>(source: &'a str, vm_or_parser: &mut VMOrCompiler) -> Value {
    let ptr_obj = Object::new_string(source, vm_or_parser);
    Value::Obj(ptr_obj as *mut Object)
}

pub fn copy_string<'a>(source: &str, vm_or_parser: &mut VMOrCompiler) -> Value {
    create_string_value(source, vm_or_parser)
}

pub fn concatenate_strings(a: &str, b: &str, vm_or_parser: &mut VMOrCompiler) -> Value {
    let mut string = a.to_string(); //need to create this allocation because HashSet's get_or_insert() method is currently unstable
    string.push_str(b);
    create_string_value(string.as_ref(), vm_or_parser)
}