use crate::chunk::{Chunk, OpCode};
use crate::gc::Gc;
use crate::object::{
    ObjBoundMethod, ObjClass, ObjClosure, ObjInstance, ObjNative, ObjString, ObjUpvalue,
};
use crate::value::{ValueType, value::*};

use std::cell::Cell;
use std::collections::HashMap;

const STACK_MAX: usize = 256;
thread_local! {
    pub static START_TIME: Cell<std::time::Instant> = Cell::new(std::time::Instant::now());
}

macro_rules! binary_op {
    ($vm: expr, $create_fn: ident, $op: tt) => {
        {
            use crate::value::value::Value;
            if !Value::is_number($vm.peek(0)?) || !Value::is_number($vm.peek(1)?) {
                $vm.runtime_error(format!("Operands must be numbers."))?;
            }
            let b = $vm.pop()?.as_number().or_else(|_| $vm.runtime_error(format!("Operand must be a number.")))?;
            let a = $vm.pop()?.as_number().or_else(|_| $vm.runtime_error(format!("Operand must be a number.")))?;
            $vm.push(Value::$create_fn(a $op b))?;
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
}

fn clock_native(_: *mut [Value]) -> Value {
    Value::number(START_TIME.with(|start_time| start_time.get().elapsed().as_secs_f64()))
}

pub struct VM {
    frames: Vec<CallFrame>,
    frame_count: usize,
    stack: [Value; STACK_MAX],
    stack_index: usize,
    globals: HashMap<Gc<ObjString>, Value>,
    pub init_string: Gc<ObjString>,
    pub open_upvalues: Option<Gc<ObjUpvalue>>,
}

impl VM {
    pub fn new() -> Self {
        let mut result = Self {
            frames: vec![],
            frame_count: 0,
            stack: std::array::from_fn(|_| Value::number(0.0).clone()),
            stack_index: 0,
            globals: HashMap::new(),
            init_string: ObjString::new("init".to_string()),
            open_upvalues: None,
        };
        result.define_native("clock", clock_native);
        result
    }

    pub fn current_chunk(&self) -> Gc<Chunk> {
        self.current_frame()
            .closure
            .borrow()
            .function
            .borrow()
            .chunk
            .clone()
    }

    pub fn reset_stack(&mut self) {
        self.stack_index = 0;
    }
    pub fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn runtime_error<T>(&mut self, msg: String) -> Result<T, InterpretError> {
        eprintln!("{}", msg);
        for i in (0..self.frame_count).rev() {
            let frame = &self.frames[i];
            let closure = frame.closure.borrow();
            let function = closure.function.borrow();
            eprint!(
                "[line {}] in ",
                function.chunk.borrow().get_line(frame.ip - 1)
            );
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
        let native = Value::native(ObjNative::new(function).into());
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
        match callee.value_type() {
            ValueType::BoundMethod => {
                let bound_method = callee.as_bound_method().unwrap();
                self.stack[self.stack_index - arg_count - 1] =
                    bound_method.borrow().receiver.clone();
                return self.call(bound_method.borrow().method.clone(), arg_count);
            }
            ValueType::Class => {
                let class = callee.as_class().unwrap();
                self.stack[self.stack_index - arg_count - 1] =
                    Value::instance(ObjInstance::new(class.clone()).into());
                if let Some(closure) = class.borrow().methods.get(&self.init_string) {
                    return self.call(closure.clone(), arg_count);
                } else if arg_count != 0 {
                    return self
                        .runtime_error(format!("Expected 0 arguments but got {}.", arg_count));
                };
                Ok(())
            }
            ValueType::Closure => return self.call(callee.as_closure().unwrap(), arg_count),
            ValueType::Native => {
                let native = callee.as_native().unwrap().borrow().function;
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
        if let Ok(instance) = receiver.as_instance() {
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
                self.push(Value::bound_method(bound_method.into()))
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
        if let Ok(class) = class.as_class() {
            if let Ok(method) = method.as_closure() {
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
        let result = std::mem::replace(&mut self.stack[self.stack_index], Value::number(0.0));
        Ok(result)
    }

    fn concatenate_strings(&mut self) -> Result<(), InterpretError> {
        let b = self.peek(0)?.clone();
        let a = self.peek(1)?.clone();
        let b = b.to_string();
        let a = a.to_string();
        if a.is_none() || b.is_none() {
            self.runtime_error("Operands must be two numbers or two strings.".to_string())?;
        }

        let new_value = concatenate_strings(a.unwrap(), b.unwrap());
        self.pop()?;
        self.pop()?;
        self.push(new_value)
    }

    fn read_operation(&mut self) -> Option<OpCode> {
        let result = self
            .current_chunk()
            .borrow()
            .read_operation(self.current_frame().ip);
        self.current_frame_mut().ip += 1;
        result
    }

    fn read_byte(&mut self) -> u8 {
        let result = self
            .current_chunk()
            .borrow()
            .read_byte(self.current_frame().ip);
        self.current_frame_mut().ip += 1;
        result
    }

    fn read_string(&mut self) -> Gc<ObjString> {
        let index = self.read_byte();
        self.current_chunk().borrow().constants[index as usize]
            .clone()
            .as_string()
            .unwrap()
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
                        let string = self.current_chunk().borrow().constants[global as usize]
                            .clone()
                            .as_string()
                            .unwrap();
                        let arg_count = self.read_byte() as usize;
                        self.invoke(string, arg_count)?;
                    }
                    OpCode::SuperInvoke => {
                        let name = self.read_string();
                        let superclass = self.pop()?.as_class().unwrap();
                        let arg_count = self.read_byte() as usize;
                        self.invoke_from_class(superclass, name, arg_count)?;
                    }
                    OpCode::Closure => {
                        let index = self.read_byte();
                        if let Ok(function) = self.current_chunk().borrow().constants
                            [index as usize]
                            .clone()
                            .as_function()
                        {
                            let closure = ObjClosure::new(function.clone());
                            self.push(Value::closure(closure.clone().into()))?;
                            for _i in 0..function.borrow().upvalue_count {
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
                                    let upvalue =
                                        parent_closure.borrow().upvalues[index as usize].clone();
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
                        let name = self.read_string();
                        let class = Value::class(ObjClass::new(name).into());
                        self.push(class)?;
                    }
                    OpCode::Inherit => {
                        let superclass = self.peek(1)?.clone().as_class().unwrap();
                        let subclass = self.peek(0)?.clone().as_class().unwrap();
                        let subclass_methods = &mut subclass.borrow_mut().methods;

                        for (name, method) in &superclass.borrow().methods {
                            subclass_methods.insert(name.clone(), method.clone());
                        }
                        self.pop()?;
                    }
                    OpCode::Method => {
                        let name = self.read_string();
                        self.define_method(name)?;
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
                        let name = self.read_string();
                        match self.globals.get(&name) {
                            None => {
                                self.runtime_error(format!("Undefined variable {}", name))?;
                            }
                            Some(value) => self.push(value.clone())?,
                        };
                    }
                    OpCode::DefineGlobal => {
                        let name = self.read_string();
                        let value = self.peek(0)?.clone();
                        self.globals.insert(name, value);
                        self.pop()?;
                    }
                    OpCode::SetGlobal => {
                        let name = self.read_string();
                        let value = self.peek(0)?.clone();
                        match self.globals.entry(name.clone()) {
                            std::collections::hash_map::Entry::Occupied(mut occupied) => {
                                *occupied.get_mut() = value;
                            }
                            std::collections::hash_map::Entry::Vacant(_) => {
                                self.runtime_error(format!("Undefined variable '{}'", name))?;
                            }
                        }
                    }
                    OpCode::Nil => self.push(Value::nil())?,
                    OpCode::False => self.push(Value::bool_(false))?,
                    OpCode::True => self.push(Value::bool_(true))?,
                    OpCode::Negate => {
                        let value = self
                            .pop()?
                            .as_number()
                            .or_else(|_| self.runtime_error(format!("Operand must be a number.")))?;
                        self.push(Value::number(-value))?;
                    }
                    OpCode::Not => {
                        let value = self.pop()?;
                        self.push(Value::bool_(value.is_falsey()))?;
                    }
                    OpCode::GetUpvalue => {
                        let slot = self.read_byte();
                        let slot =
                            self.current_frame().closure.borrow().upvalues[slot as usize].clone();
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
                        let upvalue =
                            self.current_frame().closure.borrow().upvalues[slot as usize].clone();
                        if upvalue.borrow().location.is_null() {
                            upvalue.borrow_mut().closed = value;
                        } else {
                            let location = unsafe { &mut *upvalue.borrow().location };
                            *location = value;
                        }
                    }
                    OpCode::GetProperty => {
                        let instance = self.peek(0)?.clone().as_instance();
                        if let Ok(instance) = instance {
                            let name = self.read_byte();
                            if let Ok(name) = self.current_chunk().borrow().constants[name as usize]
                                .clone()
                                .as_string()
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
                        let instance = self.peek(1)?.clone().as_instance();
                        if let Ok(instance) = instance {
                            let name = self.read_byte();
                            if let Ok(name) = self.current_chunk().borrow().constants[name as usize]
                                .clone()
                                .as_string()
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
                        let constant = self.read_byte();
                        let name = self.current_chunk().borrow().constants[constant as usize]
                            .clone()
                            .as_string()
                            .unwrap();
                        let superclass = self.pop()?.as_class().unwrap();
                        self.bind_method(superclass, name)?;
                    }
                    OpCode::Equal => {
                        let b = self.pop()?;
                        let a = self.pop()?;
                        self.push(Value::bool_(a == b))?;
                    }
                    OpCode::Greater => binary_op!(self, bool_, >),
                    OpCode::Less => binary_op!(self, bool_, <),
                    OpCode::Add => {
                        if self.peek(0)?.is_string() && self.peek(1)?.is_string() {
                            self.concatenate_strings()?;
                        } else {
                            let b = self.pop()?.as_number().or_else(|_| self.runtime_error(format!("Operands must be two numbers or two strings")))?;
                            let a = self.pop()?.as_number().or_else(|_| self.runtime_error(format!("Operands must be two numbers or two strings")))?;
                            self.push(Value::number(a + b))?;
                        }
                    }
                    OpCode::Subtract => binary_op!(self, number, -),
                    OpCode::Multiply => binary_op!(self, number, *),
                    OpCode::Divide => binary_op!(self, number, /),
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
        let function = crate::compiler::compile(source.as_str())?;
        self.push(Value::function(function.clone().into()))?;
        let closure = ObjClosure::new(function);
        self.pop()?;
        self.push(Value::closure(closure.clone().into()))?;
        self.call(closure, 0)?;
        self.run()
    }
}
