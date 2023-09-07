use std::collections::HashSet;

use crate::{
    compiler::Compiler,
    object::{ObjFunction, ObjUpvalue, Object, ObjectType, ObjClosure},
    value::Value,
    vm::VM,
};

pub enum VMOrCompiler<'a, 'b> {
    VM(&'a mut VM),
    Compiler {
        compiler: &'a mut Compiler<'b>,
        object_list: &'a mut *mut Object,
        interned_strings: &'a mut HashSet<Box<str>>,
        gray_stack: &'a mut Vec<*mut Object>,
    },
}

impl<'a, 'b> VMOrCompiler<'a, 'b> {
    pub fn objects(&mut self) -> &mut *mut Object {
        match self {
            Self::VM(vm) => vm.objects(),
            Self::Compiler { object_list, .. } => object_list,
        }
    }

    pub fn interned_strings(&mut self) -> &mut HashSet<Box<str>> {
        match self {
            Self::VM(vm) => vm.strings(),
            Self::Compiler {
                interned_strings, ..
            } => interned_strings,
        }
    }

    pub fn gray_stack(&mut self) -> &mut Vec<*mut Object> {
        match self {
            Self::VM(vm) => vm.gray_stack(),
            Self::Compiler { gray_stack, .. } => gray_stack,
        }
    }
}

pub fn allocate<T>(value: T, vm_or_parser: &mut VMOrCompiler) -> *mut T {
    #[cfg(debug_assertions)]
    collect_garbage(vm_or_parser);

    let box_ptr = Box::new(value);
    Box::into_raw(box_ptr)
}

pub fn collect_garbage(vm_or_parser: &mut VMOrCompiler) {
    #[cfg(debug_assertions)]
    println!("gc begin");

    match vm_or_parser {
        VMOrCompiler::VM(vm) => mark_roots(vm),
        VMOrCompiler::Compiler {
            compiler,
            gray_stack,
            ..
        } => compiler.mark_compiler_roots(gray_stack),
    };

    trace_references(vm_or_parser.gray_stack());

    #[cfg(debug_assertions)]
    println!("gc end");
}

pub fn mark_roots(vm: &mut VM) {
    let mut gray_stack = vec![];
    for value in vm.stack() {
        crate::allocate::mark_value(*value, &mut gray_stack);
    }

    for v in vm.global_values() {
        crate::allocate::mark_value(*v, &mut gray_stack);
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
        print!("{:?} blacken ", object);
        println!("{}", Value::Obj(object));
    }

    match unsafe { (*object).object_type } {
        ObjectType::Closure => {
            let closure = unsafe{&mut*(object as *mut ObjClosure)};
            mark_object(closure.function as *mut Object, gray_stack);
            closure.upvalues.iter().for_each(|x| mark_object(*x as *mut Object, gray_stack));
        },
        ObjectType::Function => {
            let function = unsafe{&mut*(object as *mut ObjFunction)};
            mark_object(function.name as *mut Object, gray_stack);
            function.chunk.constants.iter().for_each(|x| mark_value(*x, gray_stack));
        }
        ObjectType::Upvalue => {
            let upvalue = object as *mut ObjUpvalue;
            let value = unsafe { (*upvalue).closed };
            mark_value(value, gray_stack);
        }
        ObjectType::String | ObjectType::Native => return,
        ObjectType::_Instance => todo!(),
    }
}

fn trace_references(gray_stack: &mut Vec<*mut Object>) {
    while !gray_stack.is_empty() {
        let object = gray_stack.pop().unwrap();
        blacken_object(object, gray_stack);
    }
}

pub fn mark_value(value: Value, gray_stack: &mut Vec<*mut Object>) {
    if let Value::Obj(object) = value {
        mark_object(object, gray_stack)
    };
}

pub fn mark_object(object: *mut Object, gray_stack: &mut Vec<*mut Object>) {
    if object.is_null() || unsafe{*object}.is_marked{
        return;
    }
    #[cfg(debug_assertions)]
    {
        print!("{:?} mark", object);
        println!("{}", Object::to_string(object))
    }
    gray_stack.push(object);
    unsafe {
        (*object).is_marked = true;
    }
}
