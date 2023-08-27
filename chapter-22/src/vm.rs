use crate::chunk::{Chunk, OpCode};
use crate::object::Object;
use crate::value::Value;

use std::collections::{HashSet, HashMap};
use std::convert::Infallible;

const STACK_MAX: usize = 256;

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
#[derive(Clone, Copy)]
pub enum InterpretError {
    Compile,
    Runtime,
}

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_index: usize,
    strings: HashSet<Box<str>>,
    globals: HashMap<*const str, Value>,
    objects: *mut Object,
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free_objects();
    }
}

impl VM {
    pub fn new() -> Self {
        Self { chunk: Chunk::new(), ip: 0, stack: [Value::Number(0.0); 256], stack_index: 0, strings: HashSet::new(), globals: HashMap::new(), objects: std::ptr::null_mut(), }
    }

    pub fn reset_stack(&mut self) {
        self.stack_index = 0;
    }

    pub fn free_objects(&mut self) {
        while !self.objects.is_null() {
            let object = unsafe {Box::from_raw(self.objects)};
            let next = object.next_object();
            self.objects = next;
        }
    }

    fn runtime_error(&mut self, msg: String) -> Result<Infallible, InterpretError> {
        eprintln!("{}", msg);
        let line = self.chunk.get_line(self.ip);
        eprintln!("[line {}] in script", line);
        self.reset_stack();
        Err(InterpretError::Runtime)
    }

    pub fn peek(&mut self, index: usize) -> Result<Value, InterpretError> {
        if index > self.stack_index {
            self.runtime_error(format!("Peek index {} is greater than stack size {}.", index, self.stack_index))?;
        }
        Ok(self.stack[self.stack_index - index - 1])
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
        let new_value = crate::value::concatenate_strings(a.as_str()?, b.as_str()?, &mut self.strings,&mut self.objects);
        self.push(new_value)
    }
    
    fn read_operation(&mut self) -> Option<OpCode> {
        let result = self.chunk.read_operation(self.ip);
        self.ip += 1;
        result
    }

    fn read_byte(&mut self) -> Option<u8> {
        let result = self.chunk.read_byte(self.ip);
        self.ip += 1;
        result
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let read_op = self.read_operation();
            match read_op {
                None => return Ok(()), //must return something if there is no code
                Some(op) => {
                    match op {
                        OpCode::Return => {
                            self.free_objects();
                            return Ok(())
                        },
                        OpCode::Print => println!("{}", self.pop()?),
                        OpCode::Pop => {self.pop()?;},
                        OpCode::GetLocal => {
                            let slot = self.read_byte().unwrap();
                            self.push(self.stack[slot as usize])?;
                        },
                        OpCode::SetLocal => {
                            let slot = self.read_byte().unwrap();
                            self.stack[slot as usize] = self.peek(0)?;
                        }
                        OpCode::GetGlobal => {
                            let global = self.read_byte().unwrap();
                            if let Value::Obj(string) = self.chunk.constants[global as usize] {
                                let name = Object::as_str_ptr(string);
                                match self.globals.get(&name) {
                                    None => {self.runtime_error(format!("Undefined variable {}", unsafe{name.as_ref()}.unwrap()))?;},
                                    Some(value) => self.push(*value)?,
                                };
                            }
                            else {
                                self.runtime_error(format!("Provided global name was not a string! this is a compiler error."))?;
                            }
                        }
                        OpCode::DefineGlobal => {
                            let global = self.read_byte().unwrap();
                            if let Value::Obj(string) = self.chunk.constants[global as usize] {
                                let name = Object::as_str_ptr(string);
                                let value = self.peek(0)?;
                                self.globals.insert(name, value);
                            }
                            else {
                                self.runtime_error(format!("Provided global name was not a string! this is a compiler error."))?;
                            }
                        },
                        OpCode::SetGlobal => {
                            let global = self.read_byte().unwrap();
                            if let Value::Obj(string) = self.chunk.constants[global as usize] {
                                let name = Object::as_str_ptr(string);
                                let value = self.peek(0)?;
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
                        }
                        OpCode::Not => {
                            let value = self.pop()?;
                            self.push(Value::Bool(value.is_falsey()))?;
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
                            if index.is_none() {self.runtime_error(format!("Could not read constant value!"))?;}
                            let index = index.unwrap();
                            let value = self.chunk.constants[index as usize];
                            self.push(value)?;
                        }
                    }
                }
            }
        };
    }

    pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
        let chunk = crate::compiler::compile(source.as_str(), &mut self.strings,&mut self.objects)?;

        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }
}
