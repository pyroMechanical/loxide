use crate::{chunk::Chunk, compiler::Compiler, value::Value, vm::VM};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Object {
    pub(crate) next: *mut Object,
    pub object_type: ObjectType,
    pub is_marked: bool,
}

impl Object {
    pub fn free_object(object: *mut Object) {
        match unsafe { *object }.object_type {
            ObjectType::String => {} //handled by removing from interned string map
            ObjectType::Function => {
                let obj_function = unsafe { Box::from_raw(object as *mut ObjFunction) };
                std::mem::drop(obj_function);
            }
            ObjectType::Native => {
                let obj_native = unsafe { Box::from_raw(object as *mut ObjNative) };
                std::mem::drop(obj_native);
            }
            ObjectType::Closure => {
                let obj_closure = unsafe { Box::from_raw(object as *mut ObjClosure) };
                std::mem::drop(obj_closure);
            }
            ObjectType::Upvalue => {
                let obj_upvalue = unsafe { Box::from_raw(object as *mut ObjUpvalue) };
                std::mem::drop(obj_upvalue);
            }
            ObjectType::_Instance => todo!(),
        }
    }
    pub fn new_string(
        source: &str,
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
    ) -> *mut ObjString {
        let string = source.to_string().into_boxed_str();
        let obj_string;
        {
            obj_string = ObjString {
                object: Object {
                    next: *vm.objects(),
                    object_type: ObjectType::String,
                    is_marked: false,
                },
                string: string.clone(),
            };
        }
        let obj_string = crate::allocate::allocate(obj_string.clone(), vm, compiler);
        let ptr_str: *mut ObjString = match vm.strings().entry(string) {
            std::collections::hash_map::Entry::Occupied(occupied) => {
                unsafe { Box::from_raw(obj_string) };
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
        let obj_upvalue = ObjUpvalue {
            object: Object {
                next: *vm.objects(),
                object_type: ObjectType::Upvalue,
                is_marked: false,
            },
            location: slot,
            closed: Value::Nil,
            next: std::ptr::null_mut(),
        };
        let object = crate::allocate::allocate(obj_upvalue, vm, compiler);
        *vm.objects() = object as *mut Object;
        object
    }

    pub fn new_function(vm: &mut VM, compiler: Option<&mut Compiler>) -> *mut ObjFunction {
        let obj_function = ObjFunction {
            object: Object {
                next: *vm.objects(),
                object_type: ObjectType::Function,
                is_marked: false,
            },
            arity: 0,
            upvalue_count: 0,
            name: std::ptr::null_mut(),
            chunk: Chunk::new(),
        };
        let object = crate::allocate::allocate(obj_function, vm, compiler);
        *vm.objects() = object as *mut Object;
        object
    }

    pub fn new_closure(
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
        function: *const ObjFunction,
    ) -> *mut ObjClosure {
        let upvalue_count = unsafe { &*function }.upvalue_count;
        let obj_closure = ObjClosure {
            object: Object {
                next: *vm.objects(),
                object_type: ObjectType::Closure,
                is_marked: false,
            },
            function,
            upvalues: vec![std::ptr::null_mut(); upvalue_count],
        };
        let object = crate::allocate::allocate(obj_closure, vm, compiler);
        *vm.objects() = object as *mut Object;
        object
    }

    pub fn new_native(
        vm: &mut VM,
        compiler: Option<&mut Compiler>,
        function: fn(*mut [Value]) -> Value,
    ) -> *mut ObjNative {
        let obj_native = ObjNative {
            object: Object {
                next: *vm.objects(),
                object_type: ObjectType::Native,
                is_marked: false,
            },
            function,
        };
        let object = crate::allocate::allocate(obj_native, vm, compiler);
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
            _ => todo!(),
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
    _Instance,
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
    pub function: *const ObjFunction,
    pub upvalues: Vec<*mut ObjUpvalue>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ObjNative {
    object: Object,
    pub function: fn(*mut [Value]) -> Value,
}
