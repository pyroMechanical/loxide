use crate::{value::value::Value, gc::Trace};

pub mod operations;
pub use operations::OpCode;

#[derive(Clone, PartialEq)]
pub struct Chunk {
    pub code: Vec<u8>,
    lines: Vec<u32>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            lines: vec![],
            constants: vec![],
        }
    }

    pub fn get_line(&self, ip: usize) -> u32 {
        self.lines[ip - 1]
    }

    pub fn add_byte(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn read_operation(&self, index: usize) -> Option<OpCode> {
        if index >= self.code.len() {
            return None;
        }
        self.code[index].try_into().ok()
    }

    pub fn read_byte(&self, index: usize) -> u8 {
        *self.code.get(index).unwrap()
    }

    pub fn disassemble_instruction(&self, index: usize) -> Option<usize> {
        let op = self.read_operation(index);
        if op.is_some() {
            let line = if index != 0 && self.lines[index] == self.lines[index - 1] {
                "   |".to_string()
            } else {
                format!("{:4}", self.lines[index])
            };
            let operation = op.unwrap();
            let new_index = match operation {
                OpCode::Constant
                | OpCode::GetGlobal
                | OpCode::DefineGlobal
                | OpCode::SetGlobal
                | OpCode::GetLocal
                | OpCode::SetLocal
                | OpCode::GetUpvalue
                | OpCode::SetUpvalue
                | OpCode::Call
                | OpCode::Class
                | OpCode::GetProperty
                | OpCode::SetProperty 
                | OpCode::GetSuper
                | OpCode::Method => {
                    let constant = self.code[index + 1];
                    println!("{:04} {} {:?} {}", index, line, operation, constant);
                    index + 2
                }
                OpCode::Loop | OpCode::Jump | OpCode::JumpIfFalse => {
                    let offset1 = self.code[index + 1] as u16;
                    let offset2 = self.code[index + 2] as u16;
                    let offset = (offset1 << 8) | offset2;
                    println!("{:04} {} {:?} {}", index, line, operation, offset);
                    index + 3
                }
                OpCode::Invoke
                | OpCode::SuperInvoke => {
                    let constant = self.code[index + 1];
                    let arg_count = self.code[index + 2];
                    println!("{:04} {} {:?} ({} args) {} {}", index, line, operation, arg_count, constant, self.constants[constant as usize]);
                    index + 3
                }
                OpCode::Closure => {
                    let mut offset = index + 1;
                    let constant = self.code[offset];
                    offset += 1;
                    println!(
                        "{:04} {} {:?} {} {}",
                        index, line, operation, constant, self.constants[constant as usize]
                    );
                    if let Ok(function) = self.constants[constant as usize].clone().as_function() {
                        for _ in 0..function.borrow().upvalue_count {
                            
                            let is_local = self.code[offset];
                            offset += 1;
                            let index = self.code[offset];
                            offset += 1;
                            println!(
                                "{:04}    | {} {}",
                                offset,
                                if is_local != 0 { "local" } else { "upvalue" },
                                index
                            );
                        }
                    }
                    offset
                }
                opcode => {
                    println!("{:04} {} {:?}", index, line, opcode);
                    index + 1
                }
            };
            return Some(new_index);
        }
        return None;
    }

    pub fn disassemble(&self) {
        let mut index = Some(0);
        while index.is_some() {
            index = self.disassemble_instruction(index.unwrap());
        }
    }
}

unsafe impl Trace for Chunk {
    fn trace(&self) {
        self.constants.trace();
    }
    fn root(&self) {
        self.constants.root();
    }
    fn unroot(&self) {
        self.constants.unroot();
    }
}
