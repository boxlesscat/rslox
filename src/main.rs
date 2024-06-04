use chunk::{Chunk, OpCode};
use debug::Disassembler;
use value::{Value, ValueArray};

pub mod chunk;
pub mod debug;
pub mod value;

fn main() {
    let mut chunk = Chunk::new();
    chunk.write(OpCode::OpReturn, 1);
    chunk.write_constant(1.1, 1);
    chunk.write(OpCode::OpReturn, 2);
    chunk.write_constant(2.2, 2);
    chunk.write(OpCode::OpReturn, 3);
    chunk.write_constant(3.3, 3);
    let disassembler = Disassembler::new(&chunk);
    disassembler.disassemble_chunk("test chunk");
}
