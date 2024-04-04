use chunk::{Chunk, OpCode};
use debug::disassemble_chunk;
use value::{Value, ValueArray};

pub mod chunk;
pub mod debug;
pub mod value;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(1.2);
    chunk.write(OpCode::OpConstant, 112);
    chunk.write(constant, 112);
    chunk.write(OpCode::OpReturn, 112);
    disassemble_chunk(&chunk, "test chunk");
}
