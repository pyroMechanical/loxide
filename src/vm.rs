use crate::chunk::{Chunk, OpCode};
use crate::gc::Gc;
use crate::object::{
    ObjBoundMethod, ObjClass, ObjClosure, ObjFunction, ObjInstance, ObjNative, ObjString,
    ObjUpvalue,
};
use crate::value::{copy_string, Value};
use once_cell::sync::Lazy;

use std::cell::Ref;
use std::collections::HashMap;

const STACK_MAX: usize = 256;
const FRAMES_MAX: usize = 64;

pub static START_TIME: Lazy<std::time::Instant> = Lazy::new(|| std::time::Instant::now());

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
#[derive(Clone)]
pub struct CallFrame {
    closure: Gc<ObjClosure>,
    ip: usize,
    stack_offset: usize,
}

impl CallFrame {
    fn new(closure: Gc<ObjClosure>, stack_offset: usize) -> Self {
        Self {
            closure,
            ip: 0,
            stack_offset,
        }
    }

    pub fn closure(&self) -> Gc<ObjClosure> {
        self.closure.clone()
    }
}

fn clock_native(_: *mut [Value]) -> Value {
    Value::Number(START_TIME.elapsed().as_secs_f64())
}

pub struct VM {
    frames: Vec<CallFrame>,
    frame_count: usize,
    stack: [Value; STACK_MAX],
    stack_index: usize,
    strings: HashMap<Box<str>, Gc<ObjString>>,
    globals: HashMap<Gc<ObjString>, Value>,
    pub(crate) bytes_allocated: usize,
    pub(crate) next_gc: usize,
    pub init_string: Gc<ObjString>,
    pub open_upvalues: Option<Gc<ObjUpvalue>>,
}

impl VM {
    pub fn new() -> Self {
        let mut result = Self {
            frames: vec![],
            frame_count: 0,
            stack: std::array::from_fn(|_| Value::Number(0.0).clone()),
            stack_index: 0,
            strings: HashMap::new(),
            globals: HashMap::new(),
            bytes_allocated: 0,
            next_gc: 1024 * 1024,
            init_string: ObjString::new("init".to_string()),
            open_upvalues: None,
        };
        result.define_native("clock", clock_native);
        result
    }

    pub fn current_chunk(&self) -> Gc<Chunk> {
        self.current_frame().closure.borrow().function.borrow().chunk.clone()
    }

    pub fn globals(&self) -> &HashMap<Gc<ObjString>, Value> {
        &self.globals
    }

    pub fn reset_stack(&mut self) {
        self.stack_index = 0;
    }

    pub fn strings(&mut self) -> &mut HashMap<Box<str>, Gc<ObjString>> {
        &mut self.strings
    }
    pub fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    pub fn stack(&self) -> &[Value] {
        self.stack.split_at(self.stack_index).0
    }

    pub fn frames(&self) -> &[CallFrame] {
        self.frames.split_at(self.frame_count).0
    }

    fn runtime_error(&mut self, msg: String) -> Result<(), InterpretError> {
        eprintln!("{}", msg);
        for i in (0..self.frame_count).rev() {
            let frame = &self.frames[i];
            let closure = frame.closure.borrow();
            let function = closure.function.borrow();
            eprint!("[line {}] in ", function.chunk.borrow().get_line(frame.ip - 1));
            match &function.name {
                None => eprintln!("script"),
                Some(string) => eprintln!("{}", string.borrow().as_str()),
            };
        }
        self.reset_stack();
        Err(InterpretError::Runtime)
    }

    fn define_native(&mut self, name: &str, function: fn(*mut [Value]) -> Value) {
        let name = ObjString::new(name.to_string());
        let native = Value::Native(ObjNative::new(function));
        self.globals.insert(name, native);
    }

    pub fn peek(&mut self, index: usize) -> Result<&mut Value, InterpretError> {
        if index > self.stack_index {
            self.runtime_error(format!(
                "Peek index {} is greater than stack size {}.",
                index, self.stack_index
            ))?;
        }
        Ok(&mut self.stack[self.stack_index - index - 1])
    }

    pub fn get_value_slice(&mut self, arg_count: usize) -> Result<*mut [Value], InterpretError> {
        let stack = &mut self.stack;
        let (_, slice) = stack.split_at_mut(self.stack_index - arg_count);
        Ok(slice as *mut _)
    }

    pub fn call(&mut self, callee: Gc<ObjClosure>, arg_count: usize) -> Result<(), InterpretError> {
        let arity = callee.borrow().function.borrow().arity;
        if arg_count != arity {
            return self.runtime_error(format!(
                "Expected {} arguments but got {}",
                arg_count, arity
            ));
        }

        let frame = CallFrame::new(callee, self.stack_index - arg_count - 1);
        self.frames.push(frame);
        Ok(())
    }

    pub fn call_value(&mut self, callee: Value, arg_count: usize) -> Result<(), InterpretError> {
        match callee {
            Value::BoundMethod(bound_method) => {
                self.stack[self.stack_index - arg_count - 1] = bound_method.borrow().receiver.clone();
                return self.call(bound_method.borrow().method.clone(), arg_count);
            }
            Value::Class(class) => {
                self.stack[self.stack_index - arg_count - 1] =
                    Value::Instance(ObjInstance::new(class.clone()));
                if let Some(closure) = class.borrow().methods.get(&self.init_string) {
                    return self.call(closure.clone(), arg_count);
                } else if arg_count != 0 {
                    return self
                        .runtime_error(format!("Expected 0 arguments but got {}.", arg_count));
                };
                Ok(())
            }
            Value::Closure(closure) => return self.call(closure, arg_count),
            Value::Native(native) => {
                let native = native.borrow().function;
                let result = native(self.get_value_slice(arg_count)?);
                self.stack_index -= arg_count + 1;
                self.push(result)
            }
            _ => return self.runtime_error("Can only call functions and classes.".to_string()),
        }
    }

    fn invoke_from_class(
        &mut self,
        class: Gc<ObjClass>,
        name: Gc<ObjString>,
        arg_count: usize,
    ) -> Result<(), InterpretError> {
        match class.borrow().methods.get(&name) {
            None => return self.runtime_error(format!("Undefined property '{}'.", name)),
            Some(method) => self.call(method.clone(), arg_count),
        }
    }

    fn invoke(&mut self, name: Gc<ObjString>, arg_count: usize) -> Result<(), InterpretError> {
        let receiver = self.peek(arg_count)?.clone();
        if let Value::Instance(instance) = receiver {
            if let Some(value) = instance.borrow().fields.get(&name) {
                self.stack[self.stack_index - arg_count - 1] = value.clone();
                return self.call_value(value.clone(), arg_count);
            } else {
                return self.invoke_from_class(instance.borrow().class.clone(), name, arg_count);
            }
        } else {
            return self.runtime_error("Only instances have methods.".to_string());
        }
    }

    fn bind_method(
        &mut self,
        class: Gc<ObjClass>,
        name: Gc<ObjString>,
    ) -> Result<(), InterpretError> {
        let class_borrow = class.borrow();
        let method = class_borrow.methods.get(&name);
        match method {
            Some(method) => {
                let receiver = self.peek(0)?.clone();
                let bound_method = ObjBoundMethod::new(receiver, method.clone());
                self.pop()?;
                self.push(Value::BoundMethod(bound_method))
            }
            None => self.runtime_error(format!("Undefined property {}", name)),
        }
    }

    fn capture_upvalue(&mut self, local: *mut Value) -> Gc<ObjUpvalue> {
        let mut previous_upvalue = None;
        let mut current_upvalue = self.open_upvalues.clone();
        while let Some(upvalue) = current_upvalue.clone() {
            if upvalue.borrow().location > local {
                break;
            }
            previous_upvalue = Some(upvalue.clone());
            current_upvalue = upvalue.borrow().next.clone();
        }

        if let Some(upvalue) = current_upvalue {
            if upvalue.borrow().location == local {
                return upvalue;
            }
        }

        let created_upvalue = ObjUpvalue::new(local);
        if let Some(upvalue) = previous_upvalue {
            upvalue.borrow_mut().add_next(created_upvalue.clone());
        } else {
            self.open_upvalues = Some(created_upvalue.clone());
        };
        created_upvalue
    }

    fn close_upvalues(&mut self, last: *mut Value) {
        while let Some(open_upvalue) = self.open_upvalues.clone() {
            if open_upvalue.borrow().location < last {
                break;
            }
            let mut upvalue = open_upvalue.borrow_mut();
            upvalue.closed = unsafe { &*upvalue.location }.clone();
            upvalue.location = std::ptr::null_mut(); //rust *really* dislikes self pointers. cover this in writeup
            self.open_upvalues = upvalue.next.clone();
        }
    }

    fn define_method(&mut self, name: Gc<ObjString>) -> Result<(), InterpretError> {
        let method = self.peek(0)?.clone();
        let class = self.peek(1)?.clone();
        if let Value::Class(class) = class {
            if let Value::Closure(method) = method {
                class.borrow_mut().methods.insert(name, method);
            } else {
                self.runtime_error(format!(
                    "Provided global name was not a string! this is a compiler error."
                ))?;
            }
        }
        self.pop()?;
        Ok(())
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
        let result = std::mem::replace(&mut self.stack[self.stack_index], Value::Number(0.0));
        Ok(result)
    }

    fn concatenate_strings(&mut self) -> Result<(), InterpretError> {
        let b = self.peek(0)?.clone();
        let a = self.peek(1)?.clone();
        let b = b.as_str();
        let a = a.as_str();
        if a.is_none() || b.is_none() {
            self.runtime_error("Operands must be two numbers or two strings.".to_string())?;
        }

        let new_value = crate::value::concatenate_strings(a.unwrap(), b.unwrap(), self, None);
        self.pop()?;
        self.pop()?;
        self.push(new_value)
    }

    fn read_operation(&mut self) -> Option<OpCode> {
        let result = self.current_chunk().borrow().read_operation(self.current_frame().ip);
        self.current_frame_mut().ip += 1;
        result
    }

    fn read_byte(&mut self) -> u8 {
        let result = self.current_chunk().borrow().read_byte(self.current_frame().ip);
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
                Some(op) => match op {
                    OpCode::Jump => {
                        let offset = self.read_u16();
                        self.current_frame_mut().ip += offset as usize;
                    }
                    OpCode::JumpIfFalse => {
                        let offset = self.read_u16();
                        if self.peek(0)?.is_falsey() {
                            self.current_frame_mut().ip += offset as usize;
                        }
                    }
                    OpCode::Loop => {
                        let offset = self.read_u16();
                        self.current_frame_mut().ip -= offset as usize;
                    }
                    OpCode::Call => {
                        let arg_count = self.read_byte();
                        let callee = self.peek(arg_count as usize)?.clone();
                        self.call_value(callee, arg_count as usize)?;
                    }
                    OpCode::Invoke => {
                        let global = self.read_byte();
                        if let Value::String(string) =
                            self.current_chunk().borrow().constants[global as usize].clone()
                        {
                            let arg_count = self.read_byte() as usize;
                            self.invoke(string, arg_count)?;
                        } else {
                            self.runtime_error(format!(
                                "Provided global name was not a string! this is a compiler error."
                            ))?;
                        }
                    }
                    OpCode::SuperInvoke => {
                        let global = self.read_byte();
                        if let Value::String(name) = self.current_chunk().borrow().constants[global as usize].clone()
                        {
                            let superclass = self.pop()?;
                            if let Value::Class(superclass) = superclass {
                                let arg_count = self.read_byte() as usize;
                                self.invoke_from_class(superclass, name, arg_count)?;
                            } else {
                                self.runtime_error(format!(
                                    "Provided super was not a class! this is a compiler error."
                                ))?;
                            }
                        } else {
                            self.runtime_error(format!(
                                "Provided global name was not a string! this is a compiler error."
                            ))?;
                        }
                    }
                    OpCode::Closure => {
                        let index = self.read_byte();
                        if let Value::Function(function) =
                            self.current_chunk().borrow().constants[index as usize].clone()
                        {
                            let closure = ObjClosure::new(function.clone());
                            self.push(Value::Closure(closure.clone()))?;
                            for i in 0..function.borrow().upvalue_count {
                                let is_local = self.read_byte();
                                let index = self.read_byte();
                                if is_local != 0 {
                                    let offset = self.current_frame_mut().stack_offset;
                                    let upvalue =
                                        &mut self.stack[offset + index as usize] as *mut _;
                                    let upvalue = self.capture_upvalue(upvalue);
                                    closure.borrow_mut().upvalues.push(upvalue);
                                } else {
                                    let parent_closure = self.current_frame().closure.clone();
                                    let upvalue = parent_closure.borrow().upvalues[index as usize].clone();
                                    closure.borrow_mut().upvalues.push(upvalue);
                                }
                            }
                        } else {
                            self.runtime_error(format!(
                                "Provided value was not a function! this is a compiler error."
                            ))?;
                        }
                    }
                    OpCode::Class => {
                        let global = self.read_byte();
                        if let Value::String(name) = self.current_chunk().borrow().constants[global as usize].clone()
                        {
                            let class = Value::Class(ObjClass::new(name));
                            self.push(class)?;
                        } else {
                            self.runtime_error(format!(
                                "Provided global name was not a string! this is a compiler error."
                            ))?;
                        }
                    }
                    OpCode::Inherit => {
                        if let (Value::Class(superclass), Value::Class(subclass)) =
                            (self.peek(1)?.clone(), self.peek(0)?.clone())
                        {
                            let subclass_methods = &mut subclass.borrow_mut().methods;

                            for (name, method) in &superclass.borrow().methods {
                                subclass_methods.insert(name.clone(), method.clone());
                            }
                            self.pop()?;
                        } else {
                            self.runtime_error(format!(
                                "Provided value was not a class! this is a compiler error."
                            ))?;
                        }
                    }
                    OpCode::Method => {
                        let global = self.read_byte();
                        if let Value::String(name) = self.current_chunk().borrow().constants[global as usize].clone()
                        {
                            self.define_method(name)?;
                        } else {
                            self.runtime_error(format!(
                                "Provided global name was not a string! this is a compiler error."
                            ))?;
                        }
                    }
                    OpCode::CloseUpvalue => {
                        let last = &mut self.stack[self.stack_index - 1] as *mut _;
                        self.close_upvalues(last);
                        self.pop()?;
                    }
                    OpCode::Return => {
                        let result = self.pop()?;
                        let stack_index = self.current_frame().stack_offset;
                        let last = &mut self.stack[stack_index] as *mut _;
                        self.close_upvalues(last);
                        self.frames.pop();
                        if self.frames.len() == 0 {
                            self.pop()?;
                            return Ok(());
                        }
                        self.stack_index = stack_index;
                        self.push(result)?;
                    }
                    OpCode::Print => println!("{}", self.pop()?),
                    OpCode::Pop => {
                        self.pop()?;
                    }
                    OpCode::GetLocal => {
                        let slot = self.read_byte();
                        let offset = self.current_frame_mut().stack_offset;
                        self.push(self.stack[slot as usize + offset].clone())?;
                    }
                    OpCode::SetLocal => {
                        let slot = self.read_byte();
                        let offset = self.current_frame_mut().stack_offset;
                        self.stack[slot as usize + offset] = self.peek(0)?.clone();
                    }
                    OpCode::GetGlobal => {
                        let global = self.read_byte();
                        if let Value::String(name) = self.current_chunk().borrow().constants[global as usize].clone()
                        {
                            match self.globals.get(&name) {
                                None => {
                                    self.runtime_error(format!("Undefined variable {}", name))?;
                                }
                                Some(value) => self.push(value.clone())?,
                            };
                        } else {
                            self.runtime_error(format!(
                                "Provided global name was not a string! this is a compiler error."
                            ))?;
                        }
                    }
                    OpCode::DefineGlobal => {
                        let global = self.read_byte();
                        if let Value::String(name) = self.current_chunk().borrow().constants[global as usize].clone()
                        {
                            let value = self.peek(0)?.clone();
                            self.globals.insert(name, value);
                            self.pop()?;
                        } else {
                            self.runtime_error(format!(
                                "Provided global name was not a string! this is a compiler error."
                            ))?;
                        }
                    }
                    OpCode::SetGlobal => {
                        let global = self.read_byte();
                        if let Value::String(name) = self.current_chunk().borrow().constants[global as usize].clone()
                        {
                            let value = self.peek(0)?.clone();
                            match self.globals.entry(name.clone()) {
                                std::collections::hash_map::Entry::Occupied(mut occupied) => {
                                    *occupied.get_mut() = value;
                                }
                                std::collections::hash_map::Entry::Vacant(_) => {
                                    self.runtime_error(format!(
                                        "Undefined variable '{}'", name
                                    ))?;
                                }
                            }
                        }
                    }
                    OpCode::Nil => self.push(Value::Nil)?,
                    OpCode::False => self.push(Value::Bool(false))?,
                    OpCode::True => self.push(Value::Bool(true))?,
                    OpCode::Negate => {
                        if self.peek(0)?.is_number() {
                            let value = self.pop()?.as_number()?;
                            self.push(Value::Number(-value))?;
                        } else {
                            self.runtime_error(format!("Operand must be a number."))?;
                        }
                    }
                    OpCode::Not => {
                        let value = self.pop()?;
                        self.push(Value::Bool(value.is_falsey()))?;
                    }
                    OpCode::GetUpvalue => {
                        let slot = self.read_byte();
                        let slot = self.current_frame().closure.borrow().upvalues[slot as usize].clone();
                        let upvalue = if slot.borrow().location.is_null() {
                            slot.borrow().closed.clone()
                        } else {
                            unsafe { &*slot.borrow().location }.clone()
                        };
                        self.push(upvalue)?;
                    }
                    OpCode::SetUpvalue => {
                        let slot = self.read_byte();
                        let value = self.peek(0)?.clone();
                        let upvalue = self.current_frame().closure.borrow().upvalues[slot as usize].clone();
                        if upvalue.borrow().location.is_null() {
                            upvalue.borrow_mut().closed = value;
                        } else {
                            let location = unsafe { &mut *upvalue.borrow().location };
                            *location = value;
                        }
                    }
                    OpCode::GetProperty => {
                        let instance = self.peek(0)?.clone();
                        if let Value::Instance(instance) = instance {
                            let name = self.read_byte();
                            if let Value::String(name) =
                                self.current_chunk().borrow().constants[name as usize].clone()
                            {
                                match instance.borrow().fields.get(&name) {
                                    Some(value) => {
                                        self.pop()?;
                                        self.push(value.clone())?;
                                    }
                                    None => {
                                        self.bind_method(instance.borrow().class.clone(), name)?;
                                    }
                                }
                            }
                        } else {
                            return self
                                .runtime_error("Only instances have properties.".to_string());
                        }
                    }
                    OpCode::SetProperty => {
                        let instance = self.peek(1)?;
                        if let Value::Instance(instance) = instance.clone() {
                            let name = self.read_byte();
                            if let Value::String(name) =
                                self.current_chunk().borrow().constants[name as usize].clone()
                            {
                                instance
                                    .borrow_mut()
                                    .fields
                                    .insert(name, self.peek(0)?.clone());
                            }
                            let value = self.pop()?;
                            self.pop()?;
                            self.push(value)?;
                        }
                    }
                    OpCode::GetSuper => {
                        let name = self.read_byte();
                        if let Value::String(name) = self.current_chunk().borrow().constants[name as usize].clone() {
                            let superclass = self.pop()?;
                            if let Value::Class(superclass) = superclass {
                                self.bind_method(superclass, name)?;
                            } else {
                                todo!();
                            }
                        } else {
                            todo!();
                        }
                    }
                    OpCode::Equal => {
                        let b = self.pop()?;
                        let a = self.pop()?;
                        self.push(Value::Bool(a == b))?;
                    }
                    OpCode::Greater => binary_op!(self, Bool, >),
                    OpCode::Less => binary_op!(self, Bool, <),
                    OpCode::Add => {
                        if self.peek(0)?.is_string() && self.peek(1)?.is_string() {
                            self.concatenate_strings()?;
                        } else if self.peek(0)?.is_number() && self.peek(1)?.is_number() {
                            let b = self.pop()?.as_number()?;
                            let a = self.pop()?.as_number()?;
                            self.push(Value::Number(a + b))?;
                        } else {
                            self.runtime_error(format!(
                                "Operands must be two numbers or two strings."
                            ))?;
                        }
                    }
                    OpCode::Subtract => binary_op!(self, Number, -),
                    OpCode::Multiply => binary_op!(self, Number, *),
                    OpCode::Divide => binary_op!(self, Number, /),
                    OpCode::Constant => {
                        let index = self.read_byte();
                        let index = index;
                        let value = self.current_chunk().borrow().constants[index as usize].clone();
                        self.push(value)?;
                    }
                },
            }
        }
    }

    pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
        let function = crate::compiler::compile(source.as_str(), self)?;
        self.push(Value::Function(function.clone()))?;
        let closure = ObjClosure::new(function);
        self.pop()?;
        self.push(Value::Closure(closure.clone()))?;
        self.call(closure, 0)?;
        self.run()
    }
}
