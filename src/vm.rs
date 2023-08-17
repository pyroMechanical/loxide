use crate::chunk::{Chunk, Operation};
use crate::value::Value;

use std::rc::Rc;

use std::mem::MaybeUninit;

const STACK_MAX: usize = 256;

macro_rules! binary_op {
($vm: expr, $op: tt) => {
    {
        let b = $vm.pop();
        let a = $vm.pop();
        $vm.push(a $op b);
    }
}
}
pub enum InterpretError {
    NoChunk,
    Compile,
    Runtime,
}

pub struct VM {
    chunk: Option<Rc<Chunk>>,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_index: usize
}

impl VM {
    pub fn new() -> Self {
        Self { chunk: None, ip: 0, stack: [0.0; 256], stack_index: 0 }
    }

    pub fn push(&mut self, value: Value) {
        self.stack[self.stack_index] = value;
        self.stack_index += 1;
    }

    pub fn pop(&mut self) -> Value {
        if self.stack_index == 0 {
            panic!("Popped from an empty stack!");
        }
        self.stack_index -= 1;
        self.stack[self.stack_index]
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        if let Some(chunk) = self.chunk.clone() {
            loop {
                let read_op = chunk.read_operation(self.ip);
                match read_op {
                    None => return Ok(()), //must return something if there is no code
                    Some((new_ip, op)) => {
                        #[cfg(debug_assertions)]
                        chunk.disassemble_instruction(self.ip);
                        self.ip = new_ip;
                        match op {
                            Operation::Return => {
                                println!("{}", self.pop());
                                return Ok(())
                            },
                            Operation::Negate => {
                                let value = self.pop();
                                self.push(-value);
                            }
                            Operation::Add => binary_op!(self, +),
                            Operation::Subtract => binary_op!(self, -),
                            Operation::Multiply => binary_op!(self, *),
                            Operation::Divide => binary_op!(self, /),
                            Operation::Constant{index} => {
                                let value = chunk.constants[index as usize];
                                self.push(value);
                            }
                        }
                    }
                }
            };
        }
        else {
            return Err(InterpretError::NoChunk);
        }
    }

    pub fn interpret(&mut self, chunk: Rc<Chunk>) -> Result<(), InterpretError> {
        self.chunk = Some(chunk);
        self.ip = 0;
        self.run()
    }
}
