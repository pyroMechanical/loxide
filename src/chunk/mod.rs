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
                    Some(index) => return Some((2, Operation::Constant{index: *index}))
                }
            },
            OpCode::Return => return Some((1, Operation::Return)),
            _ => return None,
        }
    }
    
    pub fn iter(&self) -> ChunkIterator {
        ChunkIterator{chunk: self, code_index: 0}
    }

    pub fn disassemble(&self) {
        let mut iterator = self.iter();
        let mut index = iterator.index();
        let mut op = iterator.next();
        while op.is_some() {
            let line = if index != 0 && self.lines[index] == self.lines[index-1] {
                "   |".to_string()
            } else {
                format!("{:4}", self.lines[index])
            };
            println!("{:04} {} {:?}", index, line, op.unwrap());
            index = iterator.index();
            op = iterator.next();
        }
    }
}

pub struct ChunkIterator<'a>{chunk: &'a Chunk, code_index: usize}

impl<'a> ChunkIterator<'a> {
    fn index(&self) -> usize {
        self.code_index
    }
}

impl<'a> Iterator for ChunkIterator<'a> {
    type Item = Operation;

    fn next(&mut self) -> Option<Operation> {
        let (offset, op) = self.chunk.read_operation(self.code_index)?;
        self.code_index += offset;
        Some(op)
    }
}