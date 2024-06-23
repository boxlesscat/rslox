use crate::value::Value;
use std::{default::Default, fmt::Debug};

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Add,
    Subtract,
    Multiply,
    Divide,
    
    Negate,
    Not,
    
    True,
    False,
    Nil,
    
    Equal,
    Greater,
    Less,
    
    Constant(u8),

    DefineGlobal(u8),
    GetGlobal(u8),
    SetGlobal(u8),
    GetLocal(u8),
    SetLocal(u8),

    Jump(u16),
    JumpIfFalse(u16),
    Loop(u16),
    Return,
    
    Pop,
    Print,

}

#[derive(Default)]
pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write<T: Into<OpCode>>(&mut self, op_code: T, line: usize) {
        self.code.push(op_code.into());
        self.lines.push(line);
    }

    #[inline]
    pub fn code(&mut self) -> &mut [OpCode] {
        &mut self.code
    }

    #[inline]
    pub fn constants(&self) -> &[Value] {
        &self.constants
    }

    #[inline]
    pub fn lines(&self) -> &[usize] {
        &self.lines
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn write_constant(&mut self, value: Value, line: usize) -> usize {
        let index = self.add_constant(value);
        self.write(OpCode::Constant(index as u8), line);
        index
    }
}
