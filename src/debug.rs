use super::{Chunk, OpCode};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("{name}");
    let mut offset = 0;
    while offset < chunk.code().len() {
        offset = disassemble_inst(&chunk, offset);
    }
}

fn disassemble_inst(chunk: &Chunk, offset: usize) -> usize {
    print!("\n{offset:04} ");
    if offset > 0 && chunk.lines()[offset] == chunk.lines()[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines()[offset]);
    }
    let opcode = OpCode::try_from(chunk.code()[offset]);
    if opcode.is_err() {
        print!("\nUnknown OpCode {}", chunk.code()[offset]);
        return offset + 1;
    }
    let instruction = OpCode::try_from(chunk.code()[offset]).unwrap();
    print!("{:?}", instruction);
    let offset = match instruction {
        OpCode::OpConstant => constant_instruction(offset, &chunk),
        OpCode::OpReturn => simple_instruction(offset),
    };
    offset
}

#[inline]
fn constant_instruction(offset: usize, chunk: &Chunk) -> usize {
    let constant_index = chunk.code()[offset + 1] as usize;
    let constant = chunk.constants().values()[constant_index];
    let constant = format!("'{constant}'");
    print!(" {constant_index:>10} {constant:>10}");
    offset + 2
}

#[inline]
fn simple_instruction(offset: usize) -> usize {
    offset + 1
}
