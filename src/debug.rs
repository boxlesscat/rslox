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
        let len = self.chunk.code.len();
        for offset in 0..len {
            self.disassemble_inst(offset);
        }
    }

    pub fn disassemble_inst(&self, offset: usize) {
        use OpCode::*;
        print!("{offset:04} ");
        if offset > 0 && self.chunk.lines[offset] == self.chunk.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.chunk.lines[offset]);
        }
        let instruction = self.chunk.code[offset];
        match instruction {
            Add                         => self.simple_instruction("Add"),
            Subtract                    => self.simple_instruction("Subtract"),
            Multiply                    => self.simple_instruction("Multiply"),
            Divide                      => self.simple_instruction("Divide"),

            True                        => self.simple_instruction("True"),
            False                       => self.simple_instruction("False"),
            Nil                         => self.simple_instruction("Nil"),
            
            Greater                     => self.simple_instruction("Greater"),
            Less                        => self.simple_instruction("Less"),
            Equal                       => self.simple_instruction("Equal"),

            Constant(constant_index)    => self.constant_instruction("Constant", constant_index as usize),
            Negate                      => self.simple_instruction("Negate"),
            Not                         => self.simple_instruction("Not"),

            DefineGlobal(i)             => self.constant_instruction("Define Global", i as usize),
            GetGlobal(i)                => self.constant_instruction("Get Global", i as usize),
            SetGlobal(i)                => self.constant_instruction("Set Global", i as usize),

            GetLocal(i)                 => self.byte_instruction("Get Local", i as usize),
            SetLocal(i)                 => self.byte_instruction("Set Local", i as usize),
            Call(i)                     => self.byte_instruction("Call", i as usize),
            
            Jump(jump)                  => self.jump_instruction("Jump", jump as i32),
            JumpIfFalse(jump)           => self.jump_instruction("Jump If False", jump as i32),
            Loop(jump)                  => self.jump_instruction("Loop", -(jump as i32)),
            
            Return                      => self.simple_instruction("Return"),
            Pop                         => self.simple_instruction("Pop"),
            Print                       => self.simple_instruction("Print"),
        };
    }

    fn constant_instruction(&self, name: &str, constant_index: usize) {
        print!("{name:>15}");
        let constant = self.chunk.constants[constant_index].clone();
        let constant = format!("'{constant}'");
        println!(" {constant_index:>10} {constant:>10}");
    }
    
    fn simple_instruction(&self, name: &str) {
        println!("{name:>15}");
    }
    
    fn byte_instruction(&self, name: &str, slot: usize) {
        println!("{name:>15} {slot:>10}");
    }
    
    fn jump_instruction(&self, name: &str, jump: i32) {
        println!("{name:>15} {jump:>10}");
    }

}
