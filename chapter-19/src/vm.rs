use crate::chunk::{Chunk, Operation};
use crate::object::{Object};
use crate::value::Value;

use std::convert::Infallible;

use std::ptr::NonNull;
use std::mem::MaybeUninit;

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
    objects: Option<NonNull<Object>>,
}

impl Drop for VM {
    fn drop(&mut self) {
        self.free_objects();
    }
}

impl VM {
    pub fn new() -> Self {
        Self { chunk: Chunk::new(), ip: 0, stack: [Value::Number(0.0); 256], stack_index: 0, objects: None, }
    }

    pub fn reset_stack(&mut self) {
        self.stack_index = 0;
    }

    pub fn free_objects(&mut self) {
        while self.objects != None {
            let valid_object = unsafe {Box::from_raw(self.objects.unwrap().as_ptr())};
            let next = valid_object.next();
            valid_object.free();
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
        let new_value = crate::value::concatenate_strings(a.as_str()?.to_owned(), b.as_str()?, &mut self.objects)?;
        self.push(new_value)
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            let read_op = self.chunk.read_operation(self.ip);
            match read_op {
                None => return Ok(()), //must return something if there is no code
                Some((new_ip, op)) => {
                    self.ip = new_ip;
                    match op {
                        Operation::Return => {
                            println!("{}", self.pop()?);
                            self.free_objects();
                            return Ok(())
                        },
                        Operation::Nil => self.push(Value::Nil)?,
                        Operation::False => self.push(Value::Bool(false))?,
                        Operation::True => self.push(Value::Bool(true))?,
                        Operation::Negate => {
                            if self.peek(0)?.is_number() {
                                let value = self.pop()?.as_number()?;
                                self.push(Value::Number(-value))?;
                            }
                            else {
                                self.runtime_error(format!("Operand must be a number."))?;
                            }
                        }
                        Operation::Not => {
                            let value = self.pop()?;
                            self.push(Value::Bool(value.is_falsey()))?;
                        }
                        Operation::Equal => {
                            let b = self.pop()?;
                            let a = self.pop()?;
                            self.push(Value::Bool(a == b))?;
                        }
                        Operation::Greater => binary_op!(self, Bool, >),
                        Operation::Less => binary_op!(self, Bool, <),
                        Operation::Add => {
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
                        Operation::Subtract => binary_op!(self, Number, -),
                        Operation::Multiply => binary_op!(self, Number, *),
                        Operation::Divide => binary_op!(self, Number, /),
                        Operation::Constant{index} => {
                            let value = self.chunk.constants[index as usize];
                            self.push(value)?;
                        }
                    }
                }
            }
        };
    }

    pub fn interpret(&mut self, source: String) -> Result<(), InterpretError> {
        let chunk = crate::compiler::compile(source.as_str(), &mut self.objects)?;

        self.chunk = chunk;
        self.ip = 0;
        self.run()
    }
}
