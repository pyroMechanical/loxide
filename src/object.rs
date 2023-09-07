use crate::{chunk::Chunk, value::Value, allocate::VMOrCompiler};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Object {
    next: *mut Object,
    pub object_type: ObjectType,
    pub is_marked: bool,
}

impl Object {
    pub fn free_object(object: *mut Object) {
        match unsafe{*object}.object_type {
            ObjectType::String => {
                let obj_string = unsafe {Box::from_raw(object as *mut ObjString)};
                std::mem::drop(obj_string);
            },
            ObjectType::Function => {
                let obj_function = unsafe{Box::from_raw(object as *mut ObjFunction)};
                std::mem::drop(obj_function);
            },
            ObjectType::Native => {
                let obj_native = unsafe{Box::from_raw(object as *mut ObjNative)};
                std::mem::drop(obj_native);
            },
            ObjectType::Closure => {
                let obj_closure = unsafe{Box::from_raw(object as *mut ObjClosure)};
                std::mem::drop(obj_closure);
            },
            ObjectType::Upvalue => {
                let obj_upvalue = unsafe{Box::from_raw(object as *mut ObjUpvalue)};
                std::mem::drop(obj_upvalue);
            },
            ObjectType::_Instance => todo!(),
        }
    }
    pub fn new_string(source: &str, vm_or_parser: &mut VMOrCompiler) -> *mut ObjString {
        let string = source.to_string().into_boxed_str();
        let interned_string = vm_or_parser.interned_strings().get(&string);
        let ptr_str: *const str = match interned_string {
            Some(string) => &**string,
            None => {
                vm_or_parser.interned_strings().insert(string);
                &**vm_or_parser.interned_strings().get(source).unwrap()
            }
        };
        let obj_string = ObjString{object: Object{next: *vm_or_parser.objects(), object_type: ObjectType::String, is_marked: false}, string: ptr_str};
        let object = crate::allocate::allocate(obj_string, vm_or_parser);
        *vm_or_parser.objects() = object  as *mut Self;
        object
    }

    pub fn new_upvalue(mut vm_or_parser: &mut VMOrCompiler, slot: *mut Value) -> *mut ObjUpvalue {
        let obj_upvalue = ObjUpvalue{object: Object{next: *vm_or_parser.objects(), object_type: ObjectType::Upvalue, is_marked: false}, location: slot, closed: Value::Nil, next: std::ptr::null_mut()};
        let object = crate::allocate::allocate(obj_upvalue, vm_or_parser);
        *vm_or_parser.objects() = object as *mut Self;
        object
    }

    pub fn new_function(mut vm_or_parser: &mut VMOrCompiler, name: Option<*mut ObjString>) -> *mut ObjFunction {
        let obj_function = ObjFunction{object: Object{next: *vm_or_parser.objects(), object_type: ObjectType::Function, is_marked: false}, arity: 0, upvalue_count: 0, name: name.unwrap_or(std::ptr::null_mut()), chunk: Chunk::new()};
        let object = crate::allocate::allocate(obj_function, vm_or_parser);
        *vm_or_parser.objects() = object  as *mut Self;
        object
    }

    pub fn new_closure(mut vm_or_parser: &mut VMOrCompiler, function: *const ObjFunction) -> *mut ObjClosure {
        let upvalue_count = unsafe{&*function}.upvalue_count;
        let obj_closure = ObjClosure{object: Object{next: *vm_or_parser.objects(), object_type: ObjectType::Closure, is_marked: false}, function, upvalues: vec![std::ptr::null_mut(); upvalue_count]};
        let object = crate::allocate::allocate(obj_closure, vm_or_parser);
        *vm_or_parser.objects() = object as *mut Self;
        object
    }

    pub fn new_native(mut vm_or_parser: &mut VMOrCompiler, function: fn(*mut [Value]) -> Value) -> *mut ObjNative{
        let obj_native = ObjNative{object: Object{next: *vm_or_parser.objects(), object_type: ObjectType::Native, is_marked: false}, function};
        let object = crate::allocate::allocate(obj_native, vm_or_parser);
        *vm_or_parser.objects() = object as *mut Self;
        object
    }

    pub fn next_object(&self) -> *mut Object {
        self.next
    }

    pub fn to_string(object: *const Object) -> String{
        match unsafe{*object}.object_type {
            ObjectType::String => {
                let str = unsafe{(*(object as *const ObjString)).string.as_ref().unwrap()};
                str.to_owned()
            }
            ObjectType::Function => {
                let str = unsafe{(*(object as *const ObjFunction)).name.as_ref()};
                match str {
                    None => return "<script>".to_string(),
                    Some(obj_str) => Object::to_string(obj_str as *const ObjString as *const Object)
                }
            }
            ObjectType::Closure => {
                let str = unsafe{(*(*(object as *const ObjClosure)).function).name.as_ref()};
                match str {
                    None => return "<script>".to_string(),
                    Some(obj_str) => Object::to_string(obj_str as *const ObjString as *const Object)
                }
            },
            ObjectType::Native => "<native fn>".to_string(),
            ObjectType::Upvalue => {
                let upvalue = unsafe{&*(object as *const ObjUpvalue)};
                if upvalue.location.is_null() {
                    format!("{}", upvalue.closed)
                } else {
                    format!("{}", unsafe{*upvalue.location})
                }
            }
            _ => todo!()
        }
    }

    pub fn as_str_ptr(object: *mut Object) -> *const str {
        match unsafe{*object}.object_type {
            ObjectType::String => {
                unsafe{*(object as *mut ObjString)}.string
            }
            _ => todo!()
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
///contained pointers must be either null or initialized and valid.
pub enum ObjectType {
    String,
    Function,
    Native,
    Closure,
    Upvalue,
    _Instance
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ObjString {
    object: Object,
    pub string: *const str
}

impl ObjString {
    pub fn as_str(&self) -> &str {
        unsafe{self.string.as_ref()}.unwrap()
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
    pub name: *const ObjString
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
    pub function: fn(*mut [Value]) -> Value
}