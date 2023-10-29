use std::{collections::HashMap, fmt::Display};

use crate::{
    chunk::Chunk,
    gc::{Gc, Trace},
    value::Value,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    String,
    Upvalue,
    Function,
    Closure,
    Class,
    Instance,
    BoundMethod,
    Native
}

#[repr(C)]
#[derive(Clone)]
pub struct Object {
    obj_type: ObjectType
}

impl Object {
    pub fn obj_type(&self) -> ObjectType {
        self.obj_type
    }
    //cast safety: since the constructor for Object is not accessible outside this module,
    //this conversion is safe so long as we do not construct Objects outside of constructors
    pub fn as_string(&self) -> Option<&ObjString> {
        if self.obj_type() != ObjectType::String {
            return None;
        }

        Some(unsafe{std::mem::transmute(self)})
    }

    pub fn as_upvalue(&self) -> Option<&ObjUpvalue> {
        if self.obj_type() != ObjectType::Upvalue {
            return None;
        }

        Some(unsafe{std::mem::transmute(self)})
    }

    pub fn as_function(&self) -> Option<&ObjFunction> {
        if self.obj_type() != ObjectType::Function {
            return None;
        }

        Some(unsafe{std::mem::transmute(self)})
    }

    pub fn as_closure(&self) -> Option<&ObjClosure> {
        if self.obj_type() != ObjectType::Closure {
            return None;
        }

        Some(unsafe{std::mem::transmute(self)})
    }

    pub fn as_class(&self) -> Option<&ObjClass> {
        if self.obj_type() != ObjectType::Class {
            return None;
        }

        Some(unsafe{std::mem::transmute(self)})
    }

    pub fn as_instance(&self) -> Option<&ObjInstance> {
        if self.obj_type() != ObjectType::Instance {
            return None;
        }

        Some(unsafe{std::mem::transmute(self)})
    }

    pub fn as_bound_method(&self) -> Option<&ObjBoundMethod> {
        if self.obj_type() != ObjectType::BoundMethod {
            return None;
        }

        Some(unsafe{std::mem::transmute(self)})
    }

    pub fn as_native(&self) -> Option<&ObjNative> {
        if self.obj_type() != ObjectType::Native {
            return None;
        }

        Some(unsafe{std::mem::transmute(self)})
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self.obj_type(), other.obj_type()) {
            (ObjectType::String, ObjectType::String) => self.as_string() == other.as_string(),
            _ => false
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ObjectType as OT;
        match self.obj_type {
            OT::String => self.as_string().unwrap().fmt(f),
            OT::Upvalue => self.as_upvalue().unwrap().fmt(f),
            OT::Function => self.as_function().unwrap().fmt(f),
            OT::Closure => self.as_closure().unwrap().fmt(f),
            OT::Class => self.as_class().unwrap().fmt(f),
            OT::Instance => self.as_instance().unwrap().fmt(f),
            OT::BoundMethod => self.as_bound_method().unwrap().fmt(f),
            OT::Native => self.as_native().unwrap().fmt(f), 
        }
    }
}

unsafe impl Trace for Object {
    fn trace(&self) {
        //match self.obj_type {
        //    ObjectType::String => self.as_string().unwrap().trace(),
        //    ObjectType::Upvalue => upvalue.trace(),
        //    ObjectType::Function => self.as_function().expect("Object should be subset of ObjFunction").trace(),
        //    ObjectType::Closure => closure.trace(),
        //    ObjectType::Class => class.trace(),
        //    ObjectType::Instance => instance.trace(),
        //    ObjectType::BoundMethod => bound_method.trace(),
        //    ObjectType::Native => native.trace(),
        //}
    }

    fn root(&self) {
        //match self.obj_type {
        //    ObjectType::String => string.root(),
        //    ObjectType::Upvalue => upvalue.root(),
        //    ObjectType::Function => function.root(),
        //    ObjectType::Closure => closure.root(),
        //    ObjectType::Class => class.root(),
        //    ObjectType::Instance => instance.root(),
        //    ObjectType::BoundMethod => bound_method.root(),
        //    ObjectType::Native => native.root(),
        //}
    }

    fn unroot(&self) {
        //match self {
        //    Object::String(string) => string.unroot(),
        //    Object::Upvalue(upvalue) => upvalue.unroot(),
        //    Object::Function(function) => function.unroot(),
        //    Object::Closure(closure) => closure.unroot(),
        //    Object::Class(class) => class.unroot(),
        //    Object::Instance(instance) => instance.unroot(),
        //    Object::BoundMethod(bound_method) => bound_method.unroot(),
        //    Object::Native(native) => native.unroot(),
        //}
    }
}

#[repr(C)]
pub struct ObjString {
    obj: Object,
    pub string: Box<str>,
}

impl ObjString {
    pub fn new(string: String) -> Gc<ObjString> {
        let string = string.into_boxed_str();
        Gc::new(ObjString { obj: Object{obj_type: ObjectType::String}, string })
    }
    pub fn as_str(&self) -> &str {
        self.string.as_ref()
    }
}

impl Into<Gc<Object>> for Gc<ObjString> {
    fn into(self) -> Gc<Object> {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

impl TryInto<Gc<ObjString>> for Gc<Object> {
    type Error = ();
    fn try_into(self) -> Result<Gc<ObjString>, Self::Error> {
        if self.borrow().obj_type() != ObjectType::String {
            return Err(());
        }
        unsafe {
            Ok(std::mem::transmute(self))
        }
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

#[repr(C)]
#[derive(PartialEq)]
pub struct ObjUpvalue {
    obj: Object,
    pub location: *mut Value,
    pub closed: Value,
    pub next: Option<Gc<ObjUpvalue>>,
}

impl ObjUpvalue {
    pub fn new(location: *mut Value) -> Gc<ObjUpvalue> {
        Gc::new(ObjUpvalue {
            obj: Object {
                obj_type: ObjectType::Upvalue
            },
            location,
            closed: Value::Nil,
            next: None,
        })
    }

    pub fn add_next(&mut self, next: Gc<ObjUpvalue>) {
        self.next = Some(next);
    }
}

impl Into<Gc<Object>> for Gc<ObjUpvalue> {
    fn into(self) -> Gc<Object> {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

impl TryInto<Gc<ObjUpvalue>> for Gc<Object> {
    type Error = ();
    fn try_into(self) -> Result<Gc<ObjUpvalue>, Self::Error> {
        if self.borrow().obj_type() != ObjectType::Upvalue {
            return Err(());
        }
        unsafe {
            Ok(std::mem::transmute(self))
        }
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

#[repr(C)]
#[derive(PartialEq)]
pub struct ObjFunction {
    obj: Object,
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Gc<Chunk>,
    pub name: Option<Gc<ObjString>>,
}

impl ObjFunction {
    pub fn new(name: Option<Gc<ObjString>>) -> Gc<ObjFunction> {
        Gc::new(ObjFunction {
            obj: Object {
                obj_type: ObjectType::Function
            },
            arity: 0,
            upvalue_count: 0,
            name,
            chunk: Gc::new(Chunk::new()),
        })
    }
}

impl Into<Gc<Object>> for Gc<ObjFunction> {
    fn into(self) -> Gc<Object> {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

impl TryInto<Gc<ObjFunction>> for Gc<Object> {
    type Error = ();
    fn try_into(self) -> Result<Gc<ObjFunction>, Self::Error> {
        if self.borrow().obj_type() != ObjectType::Function {
            return Err(());
        }
        unsafe {
            Ok(std::mem::transmute(self))
        }
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

#[repr(C)]
#[derive(PartialEq)]
pub struct ObjClosure {
    obj: Object,
    pub function: Gc<ObjFunction>,
    pub upvalues: Vec<Gc<ObjUpvalue>>,
}

impl ObjClosure {
    pub fn new(function: Gc<ObjFunction>) -> Gc<ObjClosure> {
        Gc::new(ObjClosure {
            obj: Object {
                obj_type: ObjectType::Closure
            },
            function,
            upvalues: vec![], /*vec![std::ptr::null_mut(); upvalue_count]*/
        })
    }
}

impl Into<Gc<Object>> for Gc<ObjClosure> {
    fn into(self) -> Gc<Object> {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

impl TryInto<Gc<ObjClosure>> for Gc<Object> {
    type Error = ();
    fn try_into(self) -> Result<Gc<ObjClosure>, Self::Error> {
        if self.borrow().obj_type() != ObjectType::Closure {
            return Err(());
        }
        unsafe {
            Ok(std::mem::transmute(self))
        }
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

#[repr(C)]
#[derive(PartialEq)]
pub struct ObjClass {
    obj: Object,
    pub name: Gc<ObjString>,
    pub methods: HashMap<Gc<ObjString>, Gc<ObjClosure>>,
}

impl ObjClass {
    pub fn new(name: Gc<ObjString>) -> Gc<ObjClass> {
        Gc::new(ObjClass {
            obj: Object {
                obj_type: ObjectType::Class
            },
            name,
            methods: HashMap::new(),
        })
    }
}

impl Into<Gc<Object>> for Gc<ObjClass> {
    fn into(self) -> Gc<Object> {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

impl TryInto<Gc<ObjClass>> for Gc<Object> {
    type Error = ();
    fn try_into(self) -> Result<Gc<ObjClass>, Self::Error> {
        if self.borrow().obj_type() != ObjectType::Class {
            return Err(());
        }
        unsafe {
            Ok(std::mem::transmute(self))
        }
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

#[repr(C)]
#[derive(PartialEq)]
pub struct ObjInstance {
    obj: Object,
    pub class: Gc<ObjClass>,
    pub fields: HashMap<Gc<ObjString>, Value>,
}

impl ObjInstance {
    pub fn new(class: Gc<ObjClass>) -> Gc<ObjInstance> {
        Gc::new(ObjInstance {
            obj: Object{
                obj_type: ObjectType::Instance
            },
            class,
            fields: HashMap::new(),
        })
    }
}

impl Into<Gc<Object>> for Gc<ObjInstance> {
    fn into(self) -> Gc<Object> {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

impl TryInto<Gc<ObjInstance>> for Gc<Object> {
    type Error = ();
    fn try_into(self) -> Result<Gc<ObjInstance>, Self::Error> {
        if self.borrow().obj_type() != ObjectType::Instance {
            return Err(());
        }
        unsafe {
            Ok(std::mem::transmute(self))
        }
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

#[repr(C)]
#[derive(PartialEq)]
pub struct ObjBoundMethod {
    obj: Object,
    pub receiver: Value,
    pub method: Gc<ObjClosure>,
}

impl ObjBoundMethod {
    pub fn new(receiver: Value, method: Gc<ObjClosure>) -> Gc<ObjBoundMethod> {
        Gc::new(ObjBoundMethod {obj: Object{obj_type: ObjectType::BoundMethod}, receiver, method })
    }
}

impl Into<Gc<Object>> for Gc<ObjBoundMethod> {
    fn into(self) -> Gc<Object> {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

impl TryInto<Gc<ObjBoundMethod>> for Gc<Object> {
    type Error = ();
    fn try_into(self) -> Result<Gc<ObjBoundMethod>, Self::Error> {
        if self.borrow().obj_type() != ObjectType::BoundMethod {
            return Err(());
        }
        unsafe {
            Ok(std::mem::transmute(self))
        }
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

#[repr(C)]
#[derive(PartialEq)]
pub struct ObjNative {
    obj: Object,
    pub function: fn(*mut [Value]) -> Value,
}

impl ObjNative {
    pub fn new(function: fn(*mut [Value]) -> Value) -> Gc<ObjNative> {
        Gc::new(ObjNative { obj: Object{obj_type: ObjectType::Native}, function })
    }
}

impl Into<Gc<Object>> for Gc<ObjNative> {
    fn into(self) -> Gc<Object> {
        unsafe {
            std::mem::transmute(self)
        }
    }
}

impl TryInto<Gc<ObjNative>> for Gc<Object> {
    type Error = ();
    fn try_into(self) -> Result<Gc<ObjNative>, Self::Error> {
        if self.borrow().obj_type() != ObjectType::Native {
            return Err(());
        }
        unsafe {
            Ok(std::mem::transmute(self))
        }
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
