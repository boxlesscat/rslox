use chunk::{Chunk, OpCode};
use value::{Value, ValueArray};
use vm::VM;

pub mod chunk;
pub mod debug;
pub mod value;
pub mod vm;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write_constant(1.2, 123);
    chunk.write_constant(3.4, 123);
    chunk.write(OpCode::OpAdd, 123);
    chunk.write_constant(5.6, 123);
    chunk.write(OpCode::OpDivide, 123);
    chunk.write(OpCode::OpNegate, 123);
    chunk.write(OpCode::OpReturn, 123);
    let mut vm = VM::new(&chunk);
    vm.intepret();
}
