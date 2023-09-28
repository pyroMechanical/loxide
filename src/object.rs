use std::collections::HashMap;

use crate::{chunk::Chunk, compiler::Compiler, value::Value, vm::VM};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Object {
    pub(crate) next: *mut Object,
    pub object_type: ObjectType,
    pub is_marked: bool,
}

impl Object {
    pub fn free_object(object: *mut Object) -> usize{
        match unsafe { *object }.object_type {
            ObjectType::String => {
                let obj_string = unsafe { Box::from_raw(object as *mut ObjString) };
                std::mem::drop(obj_string);
                std::mem::size_of::<ObjString>()
            } //handled by removing from interned string map
            ObjectType::Function => {
                let obj_function = unsafe { Box::from_raw(object as *mut ObjFunction) };
                std::mem::drop(obj_function);
                std::mem::size_of::<ObjFunction>()
            }
            ObjectType::Native => {
                let obj_native = unsafe { Box::from_raw(object as *mut ObjNative) };
                std::mem::drop(obj_native);
                std::mem::size_of::<ObjNative>()
            }
            ObjectType::Closure => {
                let obj_closure = unsafe { Box::from_raw(object as *mut ObjClosure) };
                std::mem::drop(obj_closure);
                std::mem::size_of::<ObjClosure>()
            }
            ObjectType::Upvalue => {
                let obj_upvalue = unsafe { Box::from_raw(object as *mut ObjUpvalue) };
                std::mem::drop(obj_upvalue);
                std::mem::size_of::<ObjUpvalue>()
            }
            ObjectType::Class => {
                let obj_class = unsafe {Box::from_raw(object as *mut ObjClass)};
                std::mem::drop(obj_class);
                std::mem::size_of::<ObjClass>()
            }
            ObjectType::Instance => {
                let obj_instance = unsafe {Box::from_raw(object as *mut ObjInstance)};
                std::mem::drop(obj_instance);
                std::mem::size_of::<ObjInstance>()
            },
            ObjectType::BoundMethod => {
                let obj_bound_method = unsafe {Box::from_raw(object as *mut ObjBoundMethod)};
                std::mem::drop(obj_bound_method);
                std::mem::size_of::<ObjBoundMethod>()
            }
        }
    }
    pub fn new_string(
        source: &str,
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
    ) -> *mut ObjString {
        let string = source.to_string().into_boxed_str();
        let string_ptr = crate::allocate::allocate::<ObjString>(vm, compiler);
        unsafe {
            (&mut *string_ptr).write(ObjString {
                object: Object {
                    next: *vm.objects(),
                    object_type: ObjectType::String,
                    is_marked: false,
                },
                string: string.clone(),
            });
        }
        let obj_string = string_ptr as *mut ObjString;
        let ptr_str: *mut ObjString = match vm.strings().entry(string) {
            std::collections::hash_map::Entry::Occupied(occupied) => {
                unsafe { let _ = Box::from_raw(obj_string); };
                *occupied.get()
            }
            std::collections::hash_map::Entry::Vacant(vacant) => {
                let object = *vacant.insert(obj_string);
                *vm.objects() = object as *mut Object;
                object
            }
        };
        ptr_str as *mut ObjString
    }

    pub fn new_upvalue(
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
        slot: *mut Value,
    ) -> *mut ObjUpvalue {
        
        let object = crate::allocate::allocate::<ObjUpvalue>(vm, compiler);
        unsafe {
            (&mut*object).write(ObjUpvalue {
                object: Object {
                    next: *vm.objects(),
                    object_type: ObjectType::Upvalue,
                    is_marked: false,
                },
                location: slot,
                closed: Value::Nil,
                next: std::ptr::null_mut(),
            });
        }
        let object = object as *mut ObjUpvalue;
        *vm.objects() = object as *mut Object;
        object
    }

    pub fn new_function(vm: &mut VM, compiler: Option<&mut Compiler>) -> *mut ObjFunction {
        let object = crate::allocate::allocate::<ObjFunction>( vm, compiler);
        unsafe {
            (&mut*object).write(ObjFunction {
                object: Object {
                    next: *vm.objects(),
                    object_type: ObjectType::Function,
                    is_marked: false,
                },
                arity: 0,
                upvalue_count: 0,
                name: std::ptr::null_mut(),
                chunk: Chunk::new(),
            });
        }
        let object = object as *mut ObjFunction;
        *vm.objects() = object as *mut Object;
        object
    }

    pub fn new_closure(
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
        function: *mut ObjFunction,
    ) -> *mut ObjClosure {
        let upvalue_count = unsafe { &*function }.upvalue_count;
        let object = crate::allocate::allocate::<ObjClosure>(vm, compiler);
        unsafe {
            (&mut*object).write(ObjClosure {
                object: Object {
                    next: *vm.objects(),
                    object_type: ObjectType::Closure,
                    is_marked: false,
                },
                function,
                upvalues: vec![std::ptr::null_mut(); upvalue_count],
            });
        }
        let object = object as *mut ObjClosure;
        *vm.objects() = object as *mut Object;
        object
    }

    pub fn new_class(
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
        name: *mut ObjString
    ) -> *mut ObjClass {
        let class = crate::allocate::allocate(vm, compiler);
        unsafe {
            (&mut*class).write(ObjClass {
                object: Object {
                    next: *vm.objects(),
                    object_type: ObjectType::Class,
                    is_marked: false,
                },
                name,
                methods: HashMap::new()
            });
        }
        let class = class as *mut ObjClass;
        *vm.objects() = class as *mut Object;
        class
    }

    pub fn new_instance(
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
        class: *mut ObjClass
    ) -> *mut ObjInstance {
        let instance = crate::allocate::allocate(vm, compiler);
        unsafe {
            (&mut*instance).write(ObjInstance {
                object: Object {
                    next: *vm.objects(),
                    object_type: ObjectType::Instance,
                    is_marked: false,
                },
                class,
                fields: HashMap::new(),
            });
        }
        let instance = instance as *mut ObjInstance;
        *vm.objects() = instance as *mut Object;
        instance
    }

    pub fn new_bound_method(
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
        receiver: Value,
        method: *mut ObjClosure
    ) -> *mut ObjBoundMethod {
        let bound_method = crate::allocate::allocate(vm, compiler);
        unsafe {
            (&mut*bound_method).write(ObjBoundMethod {
                object: Object {
                    next: *vm.objects(),
                    object_type: ObjectType::BoundMethod,
                    is_marked: false,
                },
                receiver,
                method
            });
        }
        let bound_method = bound_method as *mut ObjBoundMethod;
        *vm.objects() = bound_method as *mut Object;
        bound_method
    }

    pub fn new_native(
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
        function: fn(*mut [Value]) -> Value,
    ) -> *mut ObjNative {
        let object = crate::allocate::allocate::<ObjNative>(vm, compiler);
        unsafe {
            (&mut*object).write(ObjNative {
                object: Object {
                    next: *vm.objects(),
                    object_type: ObjectType::Native,
                    is_marked: false,
                },
                function,
            });
        }
        let object = object as *mut ObjNative;
        *vm.objects() = object as *mut Object;
        object
    }

    pub fn next_object(&self) -> *mut Object {
        self.next
    }

    pub fn to_string(object: *const Object) -> String {
        match unsafe { *object }.object_type {
            ObjectType::String => (&unsafe { &*(object as *const ObjString) }.string).to_string(),
            ObjectType::Function => {
                let str = unsafe { (*(object as *const ObjFunction)).name.as_ref() };
                match str {
                    None => return "<script>".to_string(),
                    Some(obj_str) => {
                        Object::to_string(obj_str as *const ObjString as *const Object)
                    }
                }
            }
            ObjectType::Closure => {
                let str = unsafe { (*(*(object as *const ObjClosure)).function).name.as_ref() };
                match str {
                    None => return "<script>".to_string(),
                    Some(obj_str) => {
                        Object::to_string(obj_str as *const ObjString as *const Object)
                    }
                }
            }
            ObjectType::Native => "<native fn>".to_string(),
            ObjectType::Upvalue => {
                let upvalue = unsafe { &*(object as *const ObjUpvalue) };
                if upvalue.location.is_null() {
                    format!("{}", upvalue.closed)
                } else {
                    format!("{}", unsafe { *upvalue.location })
                }
            }
            ObjectType::Class => {
                let class = object as *const ObjClass;
                Object::to_string(unsafe{&*class}.name as *const Object)
            }
            ObjectType::Instance => {
                let instance = object as *const ObjInstance;
                let mut string = Object::to_string(unsafe{&*instance}.class as *const Object);
                string.push_str(" instance");
                string
            }
            ObjectType::BoundMethod => {
                let bound_method = object as *const ObjBoundMethod;
                Object::to_string(unsafe{&*bound_method}.method as *mut Object)
            }
        }
    }

    pub fn as_str_ptr(object: *mut Object) -> *const str {
        match unsafe { *object }.object_type {
            ObjectType::String => unsafe { &*(object as *mut ObjString) }.string.as_ref(),
            _ => todo!(),
        }
    }
}
#[derive(Clone, Copy, PartialEq, Debug)]
///contained pointers must be either null or initialized and valid.
pub enum ObjectType {
    String,
    Function,
    Native,
    Closure,
    Upvalue,
    Class,
    Instance,
    BoundMethod
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ObjString {
    object: Object,
    pub string: Box<str>,
}

impl ObjString {
    pub fn as_str(&self) -> &str {
        self.string.as_ref()
    }
    pub fn is_marked(&self) -> bool {
        self.object.is_marked
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ObjUpvalue {
    object: Object,
    pub location: *mut Value,
    pub closed: Value,
    pub next: *mut ObjUpvalue,
}

#[repr(C)]
#[derive(Clone)]
pub struct ObjFunction {
    object: Object,
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Chunk,
    pub name: *const ObjString,
}

#[repr(C)]
#[derive(Clone)]
pub struct ObjClosure {
    object: Object,
    pub function: *mut ObjFunction,
    pub upvalues: Vec<*mut ObjUpvalue>,
}

#[repr(C)]
#[derive(Clone)]
pub struct ObjClass {
    object: Object,
    pub name: *mut ObjString,
    pub methods: HashMap<*mut ObjString,  Value>,
}

#[repr(C)]
#[derive(Clone)]
pub struct ObjInstance {
    object: Object,
    pub class: *mut ObjClass,
    pub fields: HashMap<*mut ObjString, Value>
}

#[repr(C)]
#[derive(Clone)]
pub struct ObjBoundMethod {
    object: Object,
    pub receiver: Value,
    pub method: *mut ObjClosure
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ObjNative {
    object: Object,
    pub function: fn(*mut [Value]) -> Value,
}
