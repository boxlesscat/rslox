use crate::value::Value;
use crate::value::ValueArray;
use std::{default::Default, fmt::Debug};

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Add,
    Constant(u8),
    Divide,
    Multiply,
    Negate,
    Return,
    Subtract,
}

#[derive(Default)]
pub struct Chunk {
    code: Vec<OpCode>,
    constants: ValueArray,
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
    pub fn code(&self) -> &[OpCode] {
        &self.code
    }

    #[inline]
    pub fn constants(&self) -> &ValueArray {
        &self.constants
    }

    #[inline]
    pub fn lines(&self) -> &[usize] {
        &self.lines
    }

    fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write(value);
        self.constants.values().len() - 1
    }

    pub fn write_constant(&mut self, value: Value, line: usize) -> usize {
        let index = self.add_constant(value);
        self.write(OpCode::Constant(index as u8), line);
        index
    }
}
