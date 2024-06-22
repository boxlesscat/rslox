use crate::chunk::Chunk;
use crate::chunk::OpCode;

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
            Add                         => self.simple_instruction("OP Add"),
            Subtract                    => self.simple_instruction("OP Subtract"),
            Multiply                    => self.simple_instruction("OP Multiply"),
            Divide                      => self.simple_instruction("OP Divide"),

            True                        => self.simple_instruction("OP True"),
            False                       => self.simple_instruction("OP False"),
            Nil                         => self.simple_instruction("OP Nil"),
            
            Greater                     => self.simple_instruction("OP Greater"),
            Less                        => self.simple_instruction("OP Less"),
            Equal                       => self.simple_instruction("OP Equal"),

            Constant(constant_index)    => self.constant_instruction("OP Constant", constant_index as usize),
            Negate                      => self.simple_instruction("OP Negate"),
            Not                         => self.simple_instruction("OP Not"),

            OpDefineGlobal(i)           => self.constant_instruction("OP Define Global", i as usize),
            OpGetGlobal(i)              => self.constant_instruction("OP Get Global", i as usize),
            OpSetGlobal(i)              => self.constant_instruction("OP Set Global", i as usize),

            OpGetLocal(i)               => self.byte_instruction("Op Get Local", i as usize),
            OpSetLocal(i)               => self.byte_instruction("Op Set Local", i as usize),

            Pop                         => self.simple_instruction("OP Pop"),
            Print                       => self.simple_instruction("OP Print"),
            Return                      => self.simple_instruction("OP Return"),
        };
        offset + 1
    }

    fn constant_instruction(&self, name: &str, constant_index: usize) {
        print!("{name}");
        let constant = self.chunk.constants()[constant_index].clone();
        let constant = format!("'{constant}'");
        println!(" {constant_index:>10} {constant:>10}");
    }
    
    fn simple_instruction(&self, name: &str) {
        println!("{}", name);
    }
    
    fn byte_instruction(&self, name: &str, slot: usize) {
        println!(" {name:>10} {slot:>10}");
    }

}
