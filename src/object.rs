use std::{collections::HashMap, fmt::Display};

use crate::{
    chunk::Chunk,
    gc::{Gc, Trace},
    value::Value,
};

#[derive(Clone)]
pub struct ObjString {
    pub string: Box<str>,
}

impl ObjString {
    pub fn new(string: String) -> Gc<ObjString> {
        let string = string.into_boxed_str();
        Gc::new(ObjString { string })
    }
    pub fn as_str(&self) -> &str {
        self.string.as_ref()
    }
}

impl Display for ObjString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.string.fmt(f)
    }
}

impl PartialEq for ObjString {
    fn eq(&self, other: &ObjString) -> bool {
        self.string == other.string
    }
}

impl Eq for ObjString {}

impl std::hash::Hash for ObjString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.string.hash(state);
    }
}

unsafe impl Trace for ObjString {
    fn trace(&self) {}
    fn root(&self) {}
    fn unroot(&self) {}
}

#[derive(Clone, PartialEq)]
pub struct ObjUpvalue {
    pub location: *mut Value,
    pub closed: Value,
    pub next: Option<Gc<ObjUpvalue>>,
}

impl ObjUpvalue {
    pub fn new(location: *mut Value) -> Gc<ObjUpvalue> {
        Gc::new(ObjUpvalue {
            location,
            closed: Value::Nil,
            next: None,
        })
    }

    pub fn add_next(&mut self, next: Gc<ObjUpvalue>) {
        self.next = Some(next);
    }
}

impl Display for ObjUpvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("upvalue")
    }
}

unsafe impl Trace for ObjUpvalue {
    fn trace(&self) {
        if self.location.is_null() {
            self.closed.trace();
        } else {
            unsafe {
                (&*self.location).trace();
            }
        }
        self.next.as_ref().map(|x| x.trace());
    }

    fn root(&self) {
        if self.location.is_null() {
            self.closed.root();
        } else {
            unsafe {
                (&*self.location).root();
            }
        }
        self.next.as_ref().map(|x| x.root());
    }

    fn unroot(&self) {
        if self.location.is_null() {
            self.closed.unroot();
        } else {
            unsafe {
                (&*self.location).unroot();
            }
        }
        self.next.as_ref().map(|x| x.unroot());
    }
}

#[derive(Clone, PartialEq)]
pub struct ObjFunction {
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Gc<Chunk>,
    pub name: Option<Gc<ObjString>>,
}

impl ObjFunction {
    pub fn new(name: Option<Gc<ObjString>>) -> Gc<ObjFunction> {
        Gc::new(ObjFunction {
            arity: 0,
            upvalue_count: 0,
            name,
            chunk: Gc::new(Chunk::new()),
        })
    }
}

impl Display for ObjFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.name.as_ref() {
            None => f.write_str("<script>"),
            Some(name) => name.fmt(f),
        }
    }
}

unsafe impl Trace for ObjFunction {
    fn trace(&self) {
        self.chunk.trace();
        self.name.as_ref().map(|x| x.trace());
    }
    fn root(&self) {
        self.chunk.root();
        self.name.as_ref().map(|x| x.root());
    }
    fn unroot(&self) {
        self.chunk.unroot();
        self.name.as_ref().map(|x| x.unroot());
    }
}

#[derive(Clone, PartialEq)]
pub struct ObjClosure {
    pub function: Gc<ObjFunction>,
    pub upvalues: Vec<Gc<ObjUpvalue>>,
}

impl ObjClosure {
    pub fn new(function: Gc<ObjFunction>) -> Gc<ObjClosure> {
        Gc::new(ObjClosure {
            function,
            upvalues: vec![], /*vec![std::ptr::null_mut(); upvalue_count]*/
        })
    }
}

impl Display for ObjClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.function.fmt(f)
    }
}

unsafe impl Trace for ObjClosure {
    fn trace(&self) {
        self.function.trace();
        self.upvalues.trace();
    }
    fn root(&self) {
        self.function.root();
        self.upvalues.root();
    }
    fn unroot(&self) {
        self.function.unroot();
        self.upvalues.unroot();
    }
}

#[derive(Clone, PartialEq)]
pub struct ObjClass {
    pub name: Gc<ObjString>,
    pub methods: HashMap<Gc<ObjString>, Gc<ObjClosure>>,
}

impl ObjClass {
    pub fn new(name: Gc<ObjString>) -> Gc<ObjClass> {
        Gc::new(ObjClass {
            name,
            methods: HashMap::new(),
        })
    }
}

impl Display for ObjClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

unsafe impl Trace for ObjClass {
    fn trace(&self) {
        self.name.trace();
        self.methods.trace();
    }
    fn root(&self) {
        self.name.root();
        self.methods.root();
    }
    fn unroot(&self) {
        self.name.unroot();
        self.methods.unroot();
    }
}

#[derive(Clone, PartialEq)]
pub struct ObjInstance {
    pub class: Gc<ObjClass>,
    pub fields: HashMap<Gc<ObjString>, Value>,
}

impl ObjInstance {
    pub fn new(class: Gc<ObjClass>) -> Gc<ObjInstance> {
        Gc::new(ObjInstance {
            class,
            fields: HashMap::new(),
        })
    }
}

impl Display for ObjInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut name = self.class.to_string();
        name.push_str(" instance");
        f.write_str(name.as_str())
    }
}

unsafe impl Trace for ObjInstance {
    fn trace(&self) {
        self.class.trace();
        self.fields.trace();
    }
    fn root(&self) {
        self.class.root();
        self.fields.root();
    }
    fn unroot(&self) {
        self.class.unroot();
        self.fields.unroot();
    }
}

#[derive(Clone, PartialEq)]
pub struct ObjBoundMethod {
    pub receiver: Value,
    pub method: Gc<ObjClosure>,
}

impl ObjBoundMethod {
    pub fn new(receiver: Value, method: Gc<ObjClosure>) -> Gc<ObjBoundMethod> {
        Gc::new(ObjBoundMethod { receiver, method })
    }
}

impl Display for ObjBoundMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.method.fmt(f)
    }
}

unsafe impl Trace for ObjBoundMethod {
    fn trace(&self) {
        self.receiver.trace();
        self.method.trace();
    }
    fn root(&self) {
        self.receiver.root();
        self.method.root();
    }
    fn unroot(&self) {
        self.receiver.unroot();
        self.method.unroot();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ObjNative {
    pub function: fn(*mut [Value]) -> Value,
}

impl ObjNative {
    pub fn new(function: fn(*mut [Value]) -> Value) -> Gc<ObjNative> {
        Gc::new(ObjNative { function })
    }
}

impl Display for ObjNative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<native fn>")
    }
}

unsafe impl Trace for ObjNative {
    fn trace(&self) {}
    fn root(&self) {}
    fn unroot(&self) {}
}
