use super::{Chunk, OpCode};

pub struct Disassembler<'a> {
    chunk: &'a Chunk,
}

impl <'a> Disassembler<'a> {

    pub fn new(chunk: &'a Chunk) -> Self {
        Self { chunk }
    }

    pub fn disassemble_chunk(&self, name: &str) {
        println!("{name}");
        let mut offset = 0;
        while offset < self.chunk.code().len() {
            offset = self.disassemble_inst(offset);
        }
    }

    fn disassemble_inst(&self, offset: usize) -> usize {
        print!("\n{offset:04} ");
        if offset > 0 && self.chunk.lines()[offset] == self.chunk.lines()[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.chunk.lines()[offset]);
        }
        let instruction = self.chunk.code()[offset];
        match instruction {
            OpCode::OpConstant(constant_index) => self.constant_instruction("OP Constant", constant_index as usize),
            OpCode::OpReturn => self.simple_instruction("OP Return"),
        };
        offset + 1
    }

    fn constant_instruction(&self, name: &str, constant_index: usize) {
        print!("{}", name);
        let constant = self.chunk.constants().values()[constant_index];
        let constant = format!("'{constant}'");
        print!(" {constant_index:>10} {constant:>10}");
    }
    
    fn simple_instruction(&self, name: &str) {
        print!("{}", name);
    }

}
