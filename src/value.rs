use crate::gc::{Gc, Trace};
use crate::vm::InterpretError;
use crate::object::*;
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    Object(Gc<Object>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>{
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Number(num) => write!(f, "{}", num),
            Self::Object(object) => object.borrow().fmt(f),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CastError {
    NotAnObject,
    IncorrectObjectType
}

impl Value {
    pub fn to_string(&self) -> Option<String> {
        match self {
            Value::Object(object) => object.borrow().as_string().map(|x| x.as_str().to_string()),
            _ => None
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::Object(object) => return object.borrow().obj_type() == ObjectType::String,
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

    pub fn as_string(&self) -> Result<Gc<ObjString>, CastError> {
        match self {
            Self::Object(object) => object.clone().try_into().or(Err(CastError::IncorrectObjectType)),
            _ => Err(CastError::NotAnObject)
        }
    }

    pub fn as_upvalue(&self) -> Result<Gc<ObjUpvalue>, CastError> {
        match self {
            Self::Object(object) => object.clone().try_into().or(Err(CastError::IncorrectObjectType)),
            _ => Err(CastError::NotAnObject)
        }
    }

    pub fn as_function(&self) -> Result<Gc<ObjFunction>, CastError> {
        match self {
            Self::Object(object) => object.clone().try_into().or(Err(CastError::IncorrectObjectType)),
            _ => Err(CastError::NotAnObject)
        }
    }

    pub fn as_closure(&self) -> Result<Gc<ObjClosure>, CastError> {
        match self {
            Self::Object(object) => object.clone().try_into().or(Err(CastError::IncorrectObjectType)),
            _ => Err(CastError::NotAnObject)
        }
    }

    pub fn as_class(&self) -> Result<Gc<ObjClass>, CastError> {
        match self {
            Self::Object(object) => object.clone().try_into().or(Err(CastError::IncorrectObjectType)),
            _ => Err(CastError::NotAnObject)
        }
    }

    pub fn as_instance(&self) -> Result<Gc<ObjInstance>, CastError> {
        match self {
            Self::Object(object) => object.clone().try_into().or(Err(CastError::IncorrectObjectType)),
            _ => Err(CastError::NotAnObject)
        }
    }

    pub fn as_bound_method(&self) -> Result<Gc<ObjBoundMethod>, CastError> {
        match self {
            Self::Object(object) => object.clone().try_into().or(Err(CastError::IncorrectObjectType)),
            _ => Err(CastError::NotAnObject)
        }
    }

    pub fn as_native(&self) -> Result<Gc<ObjNative>, CastError> {
        match self {
            Self::Object(object) => object.clone().try_into().or(Err(CastError::IncorrectObjectType)),
            _ => Err(CastError::NotAnObject)
        }
    }
}

unsafe impl Trace for Value {
    fn trace(&self) {
        match self {
            Value::Object(object) => object.trace(),
            _ => ()
        }
    }

    fn root(&self) {
        match self {
            Value::Object(object) => object.root(),
            _ => ()
        }
    }

    fn unroot(&self) {
        match self {
            Value::Object(object) => object.unroot(),
            _ => ()
        }
    }
}

fn create_string_value<'a>(source: String) -> Value {
    Value::Object(ObjString::new(source).into())
}

pub fn copy_string<'a>(source: &str) -> Value {
    create_string_value(source.to_string())
}

pub fn concatenate_strings(a: String, b: String) -> Value {
    let mut string = a.to_string(); //need to create this allocation because HashSet's get_or_insert() method is currently unstable
    string.push_str(&b);
    create_string_value(string)
}