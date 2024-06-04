use super::{Chunk, OpCode};

pub struct Disassembler<'a> {
    chunk: &'a Chunk,
}

impl<'a> Disassembler<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self { chunk }
    }

    pub fn disassemble_chunk(&self, name: &str) {
        println!("\n{name}\n");
        let mut offset = 0;
        while offset < self.chunk.code().len() {
            offset = self.disassemble_inst(offset);
        }
    }

    pub fn disassemble_inst(&self, offset: usize) -> usize {
        use OpCode::*;
        print!("{offset:04} ");
        if offset > 0 && self.chunk.lines()[offset] == self.chunk.lines()[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.chunk.lines()[offset]);
        }
        let instruction = self.chunk.code()[offset];
        match instruction {
            OpAdd => self.simple_instruction("OP Add"),
            OpConstant(constant_index) => {
                self.constant_instruction("OP Constant", constant_index as usize)
            }
            OpDivide => self.simple_instruction("OP Divide"),
            OpMultiply => self.simple_instruction("OP Multiply"),
            OpNegate => self.simple_instruction("OP Negate"),
            OpReturn => self.simple_instruction("OP Return"),
            OpSubtract => self.simple_instruction("OP Subtract"),
        };
        offset + 1
    }

    fn constant_instruction(&self, name: &str, constant_index: usize) {
        print!("{}", name);
        let constant = self.chunk.constants().values()[constant_index];
        let constant = format!("'{constant}'");
        println!(" {constant_index:>10} {constant:>10}");
    }

    fn simple_instruction(&self, name: &str) {
        println!("{}", name);
    }
}
