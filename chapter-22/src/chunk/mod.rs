use crate::value::Value;

pub mod operations;
pub use operations::OpCode;

#[derive(Clone)]
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<u32>,
    pub constants: Vec<Value>
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
        //since instruction pointer is at next instruction,
        //we need to subtract 1 to find the current instruction.
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

    pub fn read_operation(&self, index:usize) -> Option<OpCode> {
        if index >= self.code.len() {
            return None;
        }
        self.code[index].try_into().ok()
    }

    pub fn read_byte(&self, index:usize) -> Option<u8> {
        self.code.get(index).copied()
    }

    pub fn disassemble_instruction(&self, index: usize) -> Option<usize> {
        let op = self.read_operation(index);
        if op.is_some() {
            let line = if index != 0 && self.lines[index] == self.lines[index-1] {
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
                | OpCode::SetLocal => {
                    let constant = self.code[index + 1];
                    println!("{:04} {} {:?} {}", index, line, operation, constant);
                    index + 2
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