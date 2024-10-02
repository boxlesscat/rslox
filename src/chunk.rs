use crate::value::Value;

use std::default::Default;
use std::fmt::Debug;
use std::mem;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum OpCode {
    Add,
    Call,
    CloseUpvalue,
    Closure,
    Constant,
    DefineGlobal,
    Divide,
    Equal,
    False,
    GetGlobal,
    GetLocal,
    GetUpvalue,
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
    SetUpvalue,
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

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        unsafe {
            mem::transmute(value)
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Chunk {
    pub code:       Vec<u8>,
    pub constants:  Vec<Value>,
    pub lines:      Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write<T: Into<u8>>(&mut self, value: T, line: usize) {
        self.code.push(value.into());
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

}
