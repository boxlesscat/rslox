use crate::{chunk::{Chunk, OpCode}, value::Value};

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
        println!("{name:<16} {constant:>4} '{}'", self.chunk.constants[constant]);
        offset + 2
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        let slot = self.chunk.code[offset + 1] as usize;
        println!("{name:<16} {slot:>4}");
        offset + 2
    }

    fn jump_instruction(&self, name: &str, sign: i32, offset: usize) -> usize {
        let mut jump = (self.chunk.code[offset + 1] as i16) << 8;
        jump |= self.chunk.code[offset + 2] as i16;
        println!("{name:<16} {offset:>4} -> {}", offset as i32 + 3 + sign * jump as i32);
        offset + 3
    }

    pub fn disassemble_instruction(&self, mut offset: usize) -> usize {
        print!("{offset:04} ");
        if offset > 0 && self.chunk.lines[offset] == self.chunk.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:>4} ", self.chunk.lines[offset]);
        }
        let instruction = self.chunk.code[offset].into();
        match instruction {
            OpCode::Constant        => self.constant_instruction("CONSTANT", offset),
            
            OpCode::Nil             => self.simple_instruction("NIL", offset),
            OpCode::True            => self.simple_instruction("TRUE", offset),
            OpCode::False           => self.simple_instruction("FLASE", offset),
            OpCode::Pop             => self.simple_instruction("POP", offset),

            OpCode::GetLocal        => self.byte_instruction("GET LOCAL", offset),
            OpCode::SetLocal        => self.byte_instruction("SET LOCAL", offset),
            
            OpCode::GetGlobal       => self.constant_instruction("GET GLOBAL", offset),
            OpCode::DefineGlobal    => self.constant_instruction("DEFINE GLOBAL", offset),
            OpCode::SetGlobal       => self.constant_instruction("SET GLOBAL", offset),

            OpCode::Equal           => self.simple_instruction("EQUAL", offset),
            OpCode::Greater         => self.simple_instruction("GREATER", offset),
            OpCode::Less            => self.simple_instruction("LESS", offset),

            OpCode::Add             => self.simple_instruction("ADD", offset),
            OpCode::Subtract        => self.simple_instruction("SUBTRACT", offset),
            OpCode::Multiply        => self.simple_instruction("MULTIPLY", offset),
            OpCode::Divide          => self.simple_instruction("DIVIDE", offset),
            OpCode::Not             => self.simple_instruction("NOT", offset),
            OpCode::Negate          => self.simple_instruction("NEGATE", offset),

            OpCode::Print           => self.simple_instruction("PRINT", offset),

            OpCode::Jump            => self.jump_instruction("JUMP", 1, offset),
            OpCode::JumpIfFalse     => self.jump_instruction("JUMP IF FALSE", 1, offset),
            OpCode::Loop            => self.jump_instruction("LOOP", -1, offset),

            OpCode::Call            => self.byte_instruction("CALL", offset),
            OpCode::Closure         => {
                offset += 1;
                let constant = self.chunk.code[offset] as usize;
                offset += 1;
                println!("{:<16} {:>4} {}", "CLOSURE", constant, self.chunk.constants[constant]);
                
                if let Value::Function(function) = self.chunk.constants[constant].clone() {
                    for _ in 0..function.upvalue_count {
                        let is_local = self.chunk.code[offset];
                        offset += 1;
                        let index = self.chunk.code[offset];
                        offset += 1;
                        println!("{:04}    |                       {} {}", offset - 2, if is_local as u8 != 0 {"local"} else {"upvalue"}, index as u8)
                    }
                }

                offset
            }

            OpCode::CloseUpvalue    => self.simple_instruction("CLOSE UPVALUE", offset),

            OpCode::GetUpvalue      => self.byte_instruction("GET UPVALUE", offset),
            OpCode::SetUpvalue      => self.byte_instruction("GET SETVALUE", offset),

            OpCode::Return          => self.simple_instruction("RETURN", offset),
            #[allow(unreachable_patterns)]
            _                       => {
                println!("Unknown opcode");
                offset + 1
            }
            
        }
    }

}
