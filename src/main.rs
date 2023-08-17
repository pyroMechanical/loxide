use std::env::args;
use std::rc::Rc;

mod chunk;
mod value;
mod vm;

fn main() {
    let mut vm = vm::VM::new();
    let mut chunk = chunk::Chunk::new();
    chunk.add_op(chunk::OpCode::Constant, 123);
    let constant = chunk.add_constant(1.2);
    chunk.add_byte(constant, 123);

    chunk.add_op(chunk::OpCode::Constant, 123);
    let constant = chunk.add_constant(3.4);
    chunk.add_byte(constant, 123);

    chunk.add_op(chunk::OpCode::Add, 123);

    chunk.add_op(chunk::OpCode::Constant, 123);
    let constant = chunk.add_constant(5.6);
    chunk.add_byte(constant, 123);

    chunk.add_op(chunk::OpCode::Divide, 123);

    chunk.add_op(chunk::OpCode::Negate, 123);
    chunk.add_op(chunk::OpCode::Return, 123);
    let chunk = Rc::new(chunk);
    vm.interpret(chunk);
    //chunk.disassemble();
}
