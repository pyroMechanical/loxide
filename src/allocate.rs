use std::mem::MaybeUninit;

use crate::{
    compiler::Compiler,
    object::{
        ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjString, ObjUpvalue,
        Object, ObjectType,
    },
    value::Value,
    vm::VM,
};

pub fn allocate<T>(vm: &mut VM, compiler: Option<&mut Compiler>) -> *mut MaybeUninit<T> {
    #[cfg(debug_assertions)]
    {
        //collect_garbage(vm, compiler);
    }
    #[cfg(not(debug_assertions))]
    if vm.bytes_allocated > vm.next_gc {
        collect_garbage(vm, compiler);
    }

    let box_ptr = Box::new(MaybeUninit::uninit());
    let allocation = Box::into_raw(box_ptr);
    vm.bytes_allocated += std::mem::size_of::<T>();
    allocation
}

pub fn collect_garbage(vm: &mut VM, compiler: Option<&mut Compiler>) {
    #[cfg(debug_assertions)]
    {
        //println!("gc begin");
    }

    mark_roots(vm);

    if let Some(compiler) = compiler {
        compiler.mark_compiler_roots(vm.gray_stack());
    }

    trace_references(vm.gray_stack());
    remove_white_strings(vm);
    sweep(vm);

    #[cfg(debug_assertions)]
    {
        //println!("gc end");
    }
}

pub fn mark_roots(vm: &mut VM) {
    let mut gray_stack = vec![];

    mark_object(vm.init_string as *mut Object, &mut gray_stack);
    for value in vm.stack() {
        crate::allocate::mark_value(*value, &mut gray_stack);
    }

    for (string, value) in vm.global_values() {
        crate::allocate::mark_object(*string as *mut Object, &mut gray_stack);
        crate::allocate::mark_value(*value, &mut gray_stack);
    }

    for frame in vm.frames() {
        mark_object(frame.closure() as *mut Object, &mut gray_stack);
    }

    let mut upvalue = vm.open_upvalues;
    while !upvalue.is_null() {
        mark_object(upvalue as *mut Object, &mut gray_stack);
        upvalue = unsafe { (*upvalue).next };
    }
    *vm.gray_stack() = gray_stack;
}

fn blacken_object(object: *mut Object, gray_stack: &mut Vec<*mut Object>) {
    #[cfg(debug_assertions)]
    {
        //print!("{:?} blacken ", object);
        //println!("{}", Value::Obj(object));
    }

    match unsafe { (*object).object_type } {
        ObjectType::Closure => {
            let closure = unsafe { &mut *(object as *mut ObjClosure) };
            mark_object(closure.function as *mut Object, gray_stack);
            closure
                .upvalues
                .iter()
                .for_each(|x| mark_object(*x as *mut Object, gray_stack));
        }
        ObjectType::Function => {
            let function = unsafe { &mut *(object as *mut ObjFunction) };
            mark_object(function.name as *mut Object, gray_stack);
            function
                .chunk
                .constants
                .iter()
                .for_each(|x| mark_value(*x, gray_stack));
        }
        ObjectType::Upvalue => {
            let upvalue = object as *mut ObjUpvalue;
            let value = unsafe { (*upvalue).closed };
            mark_value(value, gray_stack);
        }
        ObjectType::Class => {
            let class = object as *mut ObjClass;
            mark_object(unsafe { &*class }.name as *mut Object, gray_stack);
            for (string, value) in &unsafe { &*class }.methods {
                mark_object(*string as *mut Object, gray_stack);
                mark_value(*value, gray_stack);
            }
        }
        ObjectType::Instance => {
            let instance = object as *mut ObjInstance;
            mark_object(unsafe { &*instance }.class as *mut Object, gray_stack);
            for (string, value) in &unsafe { &*instance }.fields {
                mark_object(*string as *mut Object, gray_stack);
                mark_value(*value, gray_stack);
            }
        }
        ObjectType::BoundMethod => {
            let bound_method = object as *mut ObjBoundMethod;
            mark_value(unsafe { &*bound_method }.receiver, gray_stack);
            mark_object(unsafe { &*bound_method }.method as *mut Object, gray_stack);
        }
        ObjectType::String | ObjectType::Native => return,
    }
}

fn trace_references(gray_stack: &mut Vec<*mut Object>) {
    while !gray_stack.is_empty() {
        let object = gray_stack.pop().unwrap();
        blacken_object(object, gray_stack);
    }
}

fn remove_white_strings(vm: &mut VM) {
    let start = vm.strings().len();
    vm.strings().retain(|_, x| unsafe { &**x }.is_marked());
    let end = vm.strings().len();
    vm.bytes_allocated -= std::mem::size_of::<ObjString>() * (start - end);
}

fn sweep(vm: &mut VM) {
    let mut previous = std::ptr::null_mut();
    let mut object = *vm.objects();
    while !object.is_null() {
        //println!("object: {:?}, previous: {:?}", object, previous);
        if unsafe { &*object }.is_marked {
            unsafe {
                (*object).is_marked = false;
            }
            previous = object;
            object = unsafe { &*object }.next_object();
        } else {
            let unreached = object;
            object = unsafe { &*object }.next_object();
            if !previous.is_null() {
                unsafe {
                    (&mut *previous).next = object;
                }
            } else {
                *vm.objects() = object;
            }
            println!("{:?}", unreached);
            vm.bytes_allocated -= Object::free_object(unreached);
        }
    }
}

pub fn mark_value(value: Value, gray_stack: &mut Vec<*mut Object>) {
    if let Value::Obj(object) = value {
        mark_object(object, gray_stack)
    };
}

pub fn mark_object(object: *mut Object, gray_stack: &mut Vec<*mut Object>) {
    if object.is_null() || unsafe { *object }.is_marked {
        return;
    }
    #[cfg(debug_assertions)]
    {
        //print!("{:?} mark ", object);
        //println!("{}", Object::to_string(object));
    }
    gray_stack.push(object);
    unsafe {
        (*object).is_marked = true;
    }
}
