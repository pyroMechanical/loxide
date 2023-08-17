use std::env::args;

mod chunk;
mod value;

fn main() {
    let mut chunk = chunk::Chunk::new();
    chunk.add_op(chunk::OpCode::Constant, 123);
    let constant = chunk.add_constant(value::Value{num: 1.3});
    chunk.add_byte(constant, 123);
    chunk.add_op(chunk::OpCode::Return, 123);
    chunk.disassemble();
}
