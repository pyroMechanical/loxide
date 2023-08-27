use std::collections::HashSet;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Object {
    next: *mut Object,
    pub object_type: ObjectType,
}

impl Object {
    pub fn new_string(source: &str, interned_strings: &mut HashSet<Box<str>>, objects: &mut *mut Object) -> *mut Self {
        let string = source.to_string().into_boxed_str();
        let interned_string = interned_strings.get(&string);
        let ptr_str: *const str = match interned_string {
            Some(string) => &**string,
            None => {
                interned_strings.insert(string);
                &**interned_strings.get(source).unwrap()
            }
        };
        let obj_string = ObjString{object: Object{next: *objects, object_type: ObjectType::String}, string: ptr_str};
        let object = crate::allocate::allocate(&obj_string) as *mut Self;
        *objects = object;
        object
    }

    pub fn next_object(&self) -> *mut Object {
        self.next
    }

    pub fn to_string(object: *mut Object) -> String{
        match unsafe{*object}.object_type {
            ObjectType::String => {
                let str = unsafe{(*(object as *mut ObjString)).string.as_ref().unwrap()};
                str.to_owned()
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