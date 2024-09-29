use crate::value::Value;

use std::default::Default;
use std::fmt::Debug;
use std::mem;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Add,
    Call,
    Closure,
    Constant,
    DefineGlobal,
    Divide,
    Equal,
    False,
    GetGlobal,
    GetLocal,
    Greater,
    Jump,
    JumpIfFalse,
    Less,
    Loop,
    Multiply,
    Negate,
    Nil,
    Not,
    Pop,
    Print,
    Return,
    SetGlobal,
    SetLocal,
    Subtract,
    True,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        unsafe {
            mem::transmute(value)
        }
    }
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

}
