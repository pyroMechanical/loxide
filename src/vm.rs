use crate::allocate::VMOrCompiler;
use crate::chunk::{Chunk, OpCode};
use crate::object::{Object, ObjFunction, ObjNative, ObjectType, ObjClosure, ObjUpvalue};
use crate::value::{Value, copy_string};
use once_cell::sync::Lazy;

use std::collections::{HashSet, HashMap};

const STACK_MAX: usize = 256;
const FRAMES_MAX: usize = 64;

pub static START_TIME: Lazy<std::time::Instant> = Lazy::new(|| {std::time::Instant::now()});

macro_rules! binary_op {
    ($vm: expr, $enum_variant: ident, $op: tt) => {
        {
            use crate::value::Value::*;
            if !$vm.peek(0)?.is_number() || !$vm.peek(1)?.is_number() {
                $vm.runtime_error(format!("Operands must be numbers."))?;
            }
            let b = $vm.pop()?.as_number()?;
            let a = $vm.pop()?.as_number()?;
            $vm.push($enum_variant(a $op b))?;
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum InterpretError {
    Compile,
    Runtime,
}
#[derive(Clone, Copy)]
pub struct CallFrame {
    closure: *mut ObjClosure,
    ip: usize,
    stack_offset: usize
}

impl CallFrame {
    fn new(closure: *mut ObjClosure) -> Self {
        Self{closure, ip: 0, stack_offset: 0}
    }

    pub fn closure(&self) -> *mut ObjClosure {
        self.closure
    }
}

fn clock_native(_: *mut [Value]) -> Value {
    Value::Number(START_TIME.elapsed().as_secs_f64())
}

pub struct VM {
    frames: [CallFrame; FRAMES_MAX],
    frame_count: usize,
    stack: [Value; STACK_MAX],
    stack_index: usize,
    strings: HashSet<Box<str>>,
    globals: HashMap<*const str, Value>,
    objects: *mut Object,
    pub open_upvalues: *mut ObjUpvalue,
    gray_stack: Vec<*mut Object>,
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free_objects();
    }
}

impl VM {
    pub fn new() -> Self {
        let mut result = Self {frames: [CallFrame::new(std::ptr::null_mut()); FRAMES_MAX], frame_count: 0, stack: [Value::Number(0.0); 256], stack_index: 0, strings: HashSet::new(), globals: HashMap::new(), objects: std::ptr::null_mut(), open_upvalues: std::ptr::null_mut(), gray_stack: Vec::new()};
        result.define_native("clock", clock_native);
        result
    }

    pub fn current_chunk(& self) -> & Chunk {
        &(unsafe{&*(&*self.current_frame().closure).function}.chunk)
    }

    pub fn reset_stack(&mut self) {
        self.stack_index = 0;
    }

    pub fn objects(&mut self) -> &mut *mut Object {
        &mut self.objects
    }

    pub fn strings(&mut self) -> &mut HashSet<Box<str>> {
        &mut self.strings
    }

    pub fn gray_stack(&mut self) -> &mut Vec<*mut Object> {
        &mut self.gray_stack
    }

    pub fn current_frame(&self) -> &CallFrame {
        &self.frames[self.frame_count -1]
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        &mut self.frames[self.frame_count -1]
    }

    pub fn free_objects(&mut self) {
        //todo!() make this work for all types
        while !self.objects.is_null() {
            let object = unsafe {Box::from_raw(self.objects)};
            let next = object.next_object();
            self.objects = next;
        }
    }
    
    pub fn stack(&self) -> &[Value] {
        self.stack.split_at(self.stack_index).0
    }

    pub fn frames(&self) -> &[CallFrame] {
        self.frames.split_at(self.frame_count).0
    }

    pub fn global_values(&self) -> impl Iterator<Item = &Value> {
        self.globals.values()
    }

    fn runtime_error(&mut self, msg: String) -> Result<(), InterpretError> {
        eprintln!("{}", msg);
        for i in (0..self.frame_count).rev() {
            let frame = &self.frames[i];
            let function = unsafe{&*(&*frame.closure).function};
            eprint!("[line {}] in ", function.chunk.get_line(frame.ip - 1));
            match unsafe{function.name.as_ref()} {
                None => eprintln!("script"),
                Some(string) => eprintln!("{}", string.as_str())
            };
        }
        self.reset_stack();
        Err(InterpretError::Runtime)
    }

    fn define_native(&mut self, name: &str, function: fn(*mut [Value]) -> Value){
        let name = copy_string(name, &mut VMOrCompiler::VM(self));
        self.push(name).unwrap();
        let native = Value::Obj(Object::new_native(&mut VMOrCompiler::VM(self), function) as *mut _);
        self.push(native).unwrap();
        if let Value::Obj(name) = name {
            self.globals.insert(Object::as_str_ptr(name), native);
        }
        else {
            unreachable!()
        }
        self.pop();
        self.pop();
    }

    pub fn peek(&mut self, index: usize) -> Result<&mut Value, InterpretError> {
        if index > self.stack_index {
            self.runtime_error(format!("Peek index {} is greater than stack size {}.", index, self.stack_index))?;
        }
        Ok(&mut self.stack[self.stack_index - index - 1])
    }

    pub fn get_value_slice(&mut self, arg_count: usize) -> Result<*mut [Value], InterpretError> {
        let stack = &mut self.stack;
        let (_, slice) = stack.split_at_mut(self.stack_index - arg_count);
        Ok(slice as *mut _)
    }

    pub fn call(&mut self, callee: *mut ObjClosure, arg_count: usize) -> Result<(), InterpretError> {
        let arity = unsafe{&*(&*callee).function}.arity;
        if arg_count != arity {
            return self.runtime_error(format!("Expected {} arguments but got {}", arg_count, arity))
        }

        if self.frame_count == FRAMES_MAX {
            return self.runtime_error("Stack overflow.".to_string())
        }

        let frame = &mut self.frames[self.frame_count];
        self.frame_count += 1;
        frame.closure = callee;
        frame.ip = 0;
        frame.stack_offset = self.stack_index - arg_count - 1;
        Ok(())
    }

    pub fn call_value(&mut self, callee: Value, arg_count: usize) -> Result<(), InterpretError> {
        match callee {
            Value::Obj(object) => match unsafe{*object}.object_type {
                ObjectType::Closure => {
                    return self.call(object as *mut ObjClosure, arg_count)
                }
                ObjectType::Native => {
                    let native = object as *mut ObjNative;
                    let native = unsafe{&*native}.function;
                    let result = native(self.get_value_slice(arg_count)?);
                    self.stack_index -= arg_count + 1;
                    self.push(result)
                }
                _ => return self.runtime_error("Can only call functions and classes.".to_string()),
            }
            _ => return self.runtime_error("Can only call functions and classes.".to_string())
        }
    }


    fn capture_upvalue(&mut self, local: *mut Value) -> *mut ObjUpvalue {
        let mut previous_upvalue = std::ptr::null_mut();
        let mut upvalue = self.open_upvalues;
        while !upvalue.is_null() && unsafe{&*upvalue}.location > local {
            previous_upvalue = upvalue;
            upvalue = unsafe{&*upvalue}.next;
        }

        if !upvalue.is_null() && unsafe{&*upvalue}.location == local {
            return upvalue;
        }

        let created_upvalue = Object::new_upvalue(&mut VMOrCompiler::VM(self), local);
        if previous_upvalue.is_null() {
            self.open_upvalues = created_upvalue;
        } else {
            unsafe{&mut*previous_upvalue}.next = created_upvalue;
        };
        created_upvalue
    }

    fn close_upvalues(&mut self, last: *mut Value) {
        while !self.open_upvalues.is_null() && unsafe{&*self.open_upvalues}.location > last {
            let upvalue = unsafe{&mut*self.open_upvalues};
            upvalue.closed = unsafe{*upvalue.location};
            upvalue.location = std::ptr::null_mut(); //rust *really* dislikes self pointers. cover this
            self.open_upvalues = upvalue.next;
        }
    }

    pub fn push(&mut self, value: Value) -> Result<(), InterpretError> {
        if self.stack_index >= 255 {
            self.runtime_error(format!("Stack overflow."))?;
        }
        self.stack[self.stack_index] = value;
        self.stack_index += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Value, InterpretError> {
        if self.stack_index == 0 {
            self.runtime_error(format!("Stack is empty, no value to pop."))?;
        }
        self.stack_index -= 1;
        Ok(self.stack[self.stack_index])
    }

    fn concatenate_strings(&mut self) -> Result<(), InterpretError> {
        let b = self.pop()?;
        let a = self.pop()?;
        let new_value = crate::value::concatenate_strings(a.as_str()?, b.as_str()?,  &mut VMOrCompiler::VM(self));
        self.push(new_value)
    }
    
    fn read_operation(&mut self) -> Option<OpCode> {
        let result = self.current_chunk().read_operation(self.current_frame().ip);
        self.current_frame_mut().ip += 1;
        result
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.current_chunk().read_byte(self.current_frame().ip);
        self.current_frame_mut().ip += 1;
        result
    }

    fn read_u16(&mut self) -> u16 {
        let upper = (self.read_byte() as u16) << 8;
        let lower = self.read_byte() as u16;
        upper | lower
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let read_op = self.read_operation();
            match read_op {
                None => return Ok(()), //must return something if there is no code
                Some(op) => {
                    match op {
                        OpCode::Jump => {
                            let offset = self.read_u16();
                            self.current_frame_mut().ip += offset as usize;
                        },
                        OpCode::JumpIfFalse => {
                            let offset = self.read_u16();
                            if self.peek(0)?.is_falsey() {
                                self.current_frame_mut().ip += offset as usize;
                            }
                        },
                        OpCode::Loop => {
                            let offset = self.read_u16();
                            self.current_frame_mut().ip -= offset as usize;
                        }
                        OpCode::Call => {
                            let arg_count = self.read_byte();
                            let callee = *self.peek(arg_count as usize)?;
                            self.call_value(callee, arg_count as usize)?;
                        }
                        OpCode::Closure => {
                            let index = self.read_byte();
                            if let Value::Obj(function) = self.current_chunk().constants[index as usize] {
                                let function = function as *const ObjFunction;
                                let closure = Object::new_closure(&mut VMOrCompiler::VM(self), function);
                                self.push(Value::Obj(closure as *mut Object))?;
                                for i in 0..unsafe{&*function}.upvalue_count {
                                    let is_local = self.read_byte();
                                    let index = self.read_byte();
                                    println!("upvalue index: {}", index);
                                    if is_local != 0 {
                                        let offset = self.current_frame_mut().stack_offset;
                                        let upvalue = &mut self.stack[offset + index as usize] as *mut _;
                                        let upvalue = self.capture_upvalue(upvalue);
                                        unsafe{&mut*closure}.upvalues[i] = upvalue;
                                    } else {
                                        let parent_closure = unsafe{&*self.current_frame().closure};
                                        println!("function upvalues: {}", unsafe{&*parent_closure.function}.upvalue_count);
                                        let upvalue = parent_closure.upvalues[index as usize];
                                        unsafe{&mut*closure}.upvalues[i] = upvalue;
                                    }
                                }
                            }
                            else {
                                self.runtime_error(format!("Provided value was not a function! this is a compiler error."))?;
                            }
                        },
                        OpCode::CloseUpvalue => {
                            let last = &mut self.stack[self.stack_index - 1] as *mut _;
                            self.close_upvalues(last);
                            self.pop()?;
                        }
                        OpCode::Return => {
                            let result = self.pop()?;
                            let stack_index = self.current_frame().stack_offset;
                            self.frame_count -= 1;
                            if self.frame_count == 0 {
                                self.pop()?;
                                return Ok(())
                            }
                            self.stack_index = stack_index;
                            self.push(result)?;
                        },
                        OpCode::Print => println!("{}", self.pop()?),
                        OpCode::Pop => {self.pop()?;},
                        OpCode::GetLocal => {
                            let slot = self.read_byte();
                            let offset = self.current_frame_mut().stack_offset;
                            self.push(self.stack[slot as usize + offset])?;
                        },
                        OpCode::SetLocal => {
                            let slot = self.read_byte();
                            let offset = self.current_frame_mut().stack_offset;
                            self.stack[slot as usize + offset] = *self.peek(0)?;
                        },
                        OpCode::GetGlobal => {
                            let global = self.read_byte();
                            if let Value::Obj(string) = self.current_chunk().constants[global as usize] {
                                let name = Object::as_str_ptr(string);
                                match self.globals.get(&name) {
                                    None => {self.runtime_error(format!("Undefined variable {}", unsafe{name.as_ref()}.unwrap()))?;},
                                    Some(value) => self.push(*value)?,
                                };
                            }
                            else {
                                self.runtime_error(format!("Provided global name was not a string! this is a compiler error."))?;
                            }
                        },
                        OpCode::DefineGlobal => {
                            let global = self.read_byte();
                            if let Value::Obj(string) = self.current_chunk().constants[global as usize] {
                                let name = Object::as_str_ptr(string);
                                let value = *self.peek(0)?;
                                self.globals.insert(name, value);
                                self.pop()?;
                            }
                            else {
                                self.runtime_error(format!("Provided global name was not a string! this is a compiler error."))?;
                            }
                        },
                        OpCode::SetGlobal => {
                            let global = self.read_byte();
                            if let Value::Obj(string) = self.current_chunk().constants[global as usize] {
                                let name = Object::as_str_ptr(string);
                                let value = *self.peek(0)?;
                                match self.globals.entry(name) {
                                    std::collections::hash_map::Entry::Occupied(mut occupied) => {
                                        *occupied.get_mut() = value;
                                    },
                                    std::collections::hash_map::Entry::Vacant(_) => {self.runtime_error(format!("Undefined variable '{}'", unsafe{&*name}))?;},
                                }
                            }
                        },
                        OpCode::Nil => self.push(Value::Nil)?,
                        OpCode::False => self.push(Value::Bool(false))?,
                        OpCode::True => self.push(Value::Bool(true))?,
                        OpCode::Negate => {
                            if self.peek(0)?.is_number() {
                                let value = self.pop()?.as_number()?;
                                self.push(Value::Number(-value))?;
                            }
                            else {
                                self.runtime_error(format!("Operand must be a number."))?;
                            }
                        },
                        OpCode::Not => {
                            let value = self.pop()?;
                            self.push(Value::Bool(value.is_falsey()))?;
                        },
                        OpCode::GetUpvalue => {
                            let slot = self.read_byte();
                            let slot = unsafe{&*self.current_frame().closure}.upvalues[slot as usize];
                            let upvalue = if unsafe{&*slot}.location.is_null() {
                                unsafe{*slot}.closed
                            } else {
                                unsafe{*(*slot).location}
                            };
                            self.push(upvalue)?;
                        },
                        OpCode::SetUpvalue => {
                            let slot = self.read_byte();
                            let value = *self.peek(0)?;
                            let upvalue =  unsafe{&mut*(&mut*self.current_frame().closure).upvalues[slot as usize]};
                            if upvalue.location.is_null() {
                                upvalue.closed = value;
                            } else {
                                let location = unsafe{&mut*upvalue.location};
                                *location = value;
                            }
                        }
                        OpCode::Equal => {
                            let b = self.pop()?;
                            let a = self.pop()?;
                            self.push(Value::Bool(a == b))?;
                        },
                        OpCode::Greater => binary_op!(self, Bool, >),
                        OpCode::Less => binary_op!(self, Bool, <),
                        OpCode::Add => {
                            if self.peek(0)?.is_string() && self.peek(1)?.is_string() {
                                self.concatenate_strings()?;
                            }
                            else if self.peek(0)?.is_number() && self.peek(1)?.is_number() {
                                let b = self.pop()?.as_number()?;
                                let a = self.pop()?.as_number()?;
                                self.push(Value::Number(a + b))?;
                            }
                            else {
                                self.runtime_error(format!("Operands must be two numbers or two strings."))?;
                            }
                        },
                        OpCode::Subtract => binary_op!(self, Number, -),
                        OpCode::Multiply => binary_op!(self, Number, *),
                        OpCode::Divide => binary_op!(self, Number, /),
                        OpCode::Constant => {
                            let index = self.read_byte();
                            let index = index;
                            let value = self.current_chunk().constants[index as usize];
                            self.push(value)?;
                        },
                    }
                }
            }
        };
    }

    pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
        let function = crate::compiler::compile(source.as_str(), &mut self.strings,&mut self.objects, &mut self.gray_stack)?;
        self.push(Value::Obj(function as *mut Object))?;
        let closure = Object::new_closure(&mut VMOrCompiler::VM(self), function);
        self.pop()?;
        self.push(Value::Obj(closure as *mut Object))?;
        self.call(closure, 0)?;
        self.run()
    }
}
