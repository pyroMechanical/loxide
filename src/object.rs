use std::fmt::{Display, Formatter, Error};
use std::ptr::NonNull;

use crate::vm::InterpretError;

#[derive(Clone, Copy)]
pub struct Object {
    next: Option<NonNull<Object>>,
    pub object_type: ObjectType,
}

impl Object {
    pub fn new(object_type: ObjectType, object_list: &mut Option<NonNull<Object>>, error_value: InterpretError) -> Result<NonNull<Self>, InterpretError> {
        let box_obj = Box::new(Object {
            object_type,
            next: *object_list
        });
        let ptr_obj = NonNull::new(Box::leak(box_obj)).ok_or(error_value)?;
        *object_list = Some(ptr_obj);
        Ok(ptr_obj)
    }

    pub fn next(&self) -> Option<NonNull<Object>> {
        self.next
    }
    /// must be called before dropping an Object, or else it will leak memory.
    /// cannot be implemented as a drop function, as this is only called during garbage collection
    /// and termination of the interpreter.
    pub fn free(self) {
        match self.object_type {
            ObjectType::String(str) => {
                let string = unsafe{Box::from_raw(str.as_ptr())};
                std::mem::drop(string)
            }
            _ => todo!()
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.object_type)
    }
}

#[derive(Clone, Copy)]
///contained pointers must be either null or initialized and valid.
pub enum ObjectType {
    String(NonNull<str>),
    Instance
}

impl PartialEq for ObjectType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ObjectType::String(str1), ObjectType::String(str2)) => {
                unsafe {
                    str1.as_ref() == str2.as_ref()
                }
            },
            _ => false,
        }
    }
}

impl ObjectType {
    fn as_str(&self) -> &str {
        match self {
            ObjectType::String(str) => unsafe {str.as_ref()},
            _ => todo!()
        }
    }
}

impl std::fmt::Debug for ObjectType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error>{
        match self {
            ObjectType::String(str) => write!(f, "ObjectType::String(\"{}\")", unsafe {str.as_ref()}),
            _ => todo!()
        }
    }
}

impl std::fmt::Display for ObjectType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{}", self.as_str())
    }
}