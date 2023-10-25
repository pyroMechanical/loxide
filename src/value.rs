use crate::compiler::Compiler;
use crate::gc::{Gc, Trace, GcCellRef};
use crate::vm::InterpretError;
use crate::vm::VM;
use crate::object::*;
use std::cell::Ref;
use std::fmt::{Display, Error, Formatter};

#[derive(Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(Gc<ObjString>),
    Upvalue(Gc<ObjUpvalue>),
    Function(Gc<ObjFunction>),
    Closure(Gc<ObjClosure>),
    Class(Gc<ObjClass>),
    Instance(Gc<ObjInstance>),
    BoundMethod(Gc<ObjBoundMethod>),
    Native(Gc<ObjNative>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>{
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Number(num) => write!(f, "{}", num),
            Self::String(string) => string.fmt(f),
            Self::Upvalue(upvalue) => upvalue.fmt(f),
            Self::Function(function) => function.fmt(f),
            Self::Closure(closure) => closure.fmt(f),
            Self::Class(class) => class.fmt(f),
            Self::Instance(instance) => instance.fmt(f),
            Self::BoundMethod(bound_method) => bound_method.fmt(f),
            Self::Native(native) => native.fmt(f),
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
            Self::String(_) => true,
            _ => false
        }
    }

    pub fn as_str(&self) -> Option<String> {
        match self {
            Value::String(string) => Some(string.borrow().as_str().to_string()),
            _ => None
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

unsafe impl Trace for Value {
    fn trace(&self) {
        match self {
            Value::String(string) => string.trace(),
            Value::Upvalue(upvalue) => upvalue.trace(),
            Value::Function(function) => function.trace(),
            Value::Closure(closure) => closure.trace(),
            Value::Class(class) => class.trace(),
            Value::Instance(instance) => instance.trace(),
            Value::BoundMethod(bound_method) => bound_method.trace(),
            Value::Native(native) => native.trace(),
            _ => ()
        }
    }

    fn root(&self) {
        match self {
            Value::String(string) => string.root(),
            Value::Upvalue(upvalue) => upvalue.root(),
            Value::Function(function) => function.root(),
            Value::Closure(closure) => closure.root(),
            Value::Class(class) => class.root(),
            Value::Instance(instance) => instance.root(),
            Value::BoundMethod(bound_method) => bound_method.root(),
            Value::Native(native) => native.root(),
            _ => ()
        }
    }

    fn unroot(&self) {
        match self {
            Value::String(string) => string.unroot(),
            Value::Upvalue(upvalue) => upvalue.unroot(),
            Value::Function(function) => function.unroot(),
            Value::Closure(closure) => closure.unroot(),
            Value::Class(class) => class.unroot(),
            Value::Instance(instance) => instance.unroot(),
            Value::BoundMethod(bound_method) => bound_method.unroot(),
            Value::Native(native) => native.unroot(),
            _ => ()
        }
    }
}

fn create_string_value<'a>(source: String, vm: &mut VM, compiler: Option<&mut Compiler>) -> Value {
    Value::String(ObjString::new(source))
}

pub fn copy_string<'a>(source: &str, vm: &mut VM, compiler: Option<&mut Compiler>) -> Value {
    create_string_value(source.to_string(), vm, compiler)
}

pub fn concatenate_strings(a: String, b: String, vm: &mut VM, compiler: Option<&mut Compiler>) -> Value {
    let mut string = a.to_string(); //need to create this allocation because HashSet's get_or_insert() method is currently unstable
    string.push_str(&b);
    create_string_value(string, vm, compiler)
}