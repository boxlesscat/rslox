use crate::value::Value;

use std::default::Default;
use std::fmt::Debug;


#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Add,
    Call(u8),
    Constant(u8),
    DefineGlobal(u8),
    Divide,
    Equal,
    False,
    GetGlobal(u8),
    GetLocal(u8),
    Greater,
    Jump(u16),
    JumpIfFalse(u16),
    Less,
    Loop(u16),
    Multiply,
    Negate,
    Nil,
    Not,
    Pop,
    Print,
    Return,
    SetGlobal(u8),
    SetLocal(u8),
    Subtract,
    True,
}

#[derive(Default, Debug, Clone)]
pub struct Chunk {
    pub code:       Vec<OpCode>,
    pub constants:  Vec<Value>,
    pub lines:      Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write<T: Into<OpCode>>(&mut self, op_code: T, line: usize) {
        self.code.push(op_code.into());
        self.lines.push(line);
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
