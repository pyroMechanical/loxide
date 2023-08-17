use crate::value::Value;

pub mod operations;
pub use operations::{Operation, OpCode};
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<u32>,
    constants: Vec<Value>
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            lines: vec![],
            constants: vec![],
        }
    }

    pub fn add_op(&mut self, op: OpCode, line: u32) {
        self.add_byte(op as u8, line);
    }

    pub fn add_byte(&mut self, byte: u8, line: u32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1) as u8
    }

    pub fn read_operation(&self, index: usize) -> Option<(usize, Operation)> {
        if index >= self.code.len() {
            return None;
        }
        let op_code: OpCode = self.code[index].try_into().ok()?;

        match op_code {
            OpCode::Constant => {
                let value_index = self.code.get(index + 1);
                match value_index {
                    None => return None,
                    Some(value_index) => return Some((index + 2, Operation::Constant{index: *value_index}))
                }
            },
            OpCode::Return => return Some((index + 1, Operation::Return)),
            _ => return None,
        }
    }

    pub fn disassemble(&self) {
        let mut index = 0;
        let mut op = self.read_operation(index);
        while op.is_some() {
            let line = if index != 0 && self.lines[index] == self.lines[index-1] {
                "   |".to_string()
            } else {
                format!("{:4}", self.lines[index])
            };
            let (new_index, operation) = op.unwrap();
            println!("{:04} {} {:?}", index, line, operation);
            index = new_index;
            op = self.read_operation(index);
        }
    }
}