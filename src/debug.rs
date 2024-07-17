use crate::chunk::{Chunk, OpCode};

pub struct Disassembler<'a> {
    chunk: &'a Chunk,
}

impl<'a> Disassembler<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self { chunk }
    }

    pub fn disassemble_chunk(&self, name: &str) {
        println!("== {name} ==");
        let mut offset = 0;
        let count = self.chunk.code.len();
        while offset < count {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.chunk.code[offset + 1] as usize;
        println!("{name:>16} {constant:>4} '{}'", self.chunk.constants[constant]);
        offset + 2
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        let slot = self.chunk.code[offset + 1] as usize;
        println!("{name:>16} {slot:>4}");
        offset + 2
    }

    fn jump_instruction(&self, name: &str, sign: i32, offset: usize) -> usize {
        let mut jump = (self.chunk.code[offset + 1] as i16) << 8;
        jump |= self.chunk.code[offset + 2] as i16;
        println!("{name:>16} {offset:>4} -> {}", offset as i32 + 3 + sign * jump as i32);
        offset + 3
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{offset:>4}");
        if offset > 0 && self.chunk.lines[offset] == self.chunk.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:>4}", self.chunk.lines[offset]);
        }
        let instruction = self.chunk.code[offset];
        match instruction {
            OpCode::Constant        => self.constant_instruction("Constant", offset),
            
            OpCode::Nil             => self.simple_instruction("Nil", offset),
            OpCode::True            => self.simple_instruction("True", offset),
            OpCode::False           => self.simple_instruction("False", offset),
            OpCode::Pop             => self.simple_instruction("Pop", offset),

            OpCode::GetLocal        => self.byte_instruction("Get Local", offset),
            OpCode::SetLocal        => self.byte_instruction("Set Local", offset),
            
            OpCode::GetGlobal       => self.constant_instruction("Get Global", offset),
            OpCode::DefineGlobal    => self.constant_instruction("Define Global", offset),
            OpCode::SetGlobal       => self.constant_instruction("Set Global", offset),

            OpCode::Equal           => self.simple_instruction("Equal", offset),
            OpCode::Greater         => self.simple_instruction("Greater", offset),
            OpCode::Less            => self.simple_instruction("Less", offset),

            OpCode::Add             => self.simple_instruction("Add", offset),
            OpCode::Subtract        => self.simple_instruction("Subtract", offset),
            OpCode::Multiply        => self.simple_instruction("Multiply", offset),
            OpCode::Divide          => self.simple_instruction("Divide", offset),
            OpCode::Not             => self.simple_instruction("Not", offset),
            OpCode::Negate          => self.simple_instruction("Negate", offset),

            OpCode::Print           => self.simple_instruction("Print", offset),

            OpCode::Jump            => self.jump_instruction("Jump", 1, offset),
            OpCode::JumpIfFalse     => self.jump_instruction("Jump If False", 1, offset),
            OpCode::Loop            => self.jump_instruction("Loop", -1, offset),

            OpCode::Call            => self.byte_instruction("Call", offset),

            OpCode::Return          => self.simple_instruction("Return", offset),
            #[allow(unreachable_patterns)]
            _                       => {
                println!("Unknown opcode");
                return offset + 1
            }
            
        }
    }

}
