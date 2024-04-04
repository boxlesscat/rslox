use super::{Value, ValueArray};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::{default::Default, fmt::Debug};

#[derive(IntoPrimitive, Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum OpCode {
    OpConstant,
    OpReturn,
}

#[derive(Default)]
pub struct Chunk {
    code: Vec<u8>,
    constants: ValueArray,
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn write<T: Into<u8>>(&mut self, byte: T, line: usize) {
        self.code.push(byte.into());
        self.lines.push(line);
    }

    #[inline]
    pub fn code(&self) -> &[u8] {
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

    #[inline]
    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.write(value);
        // If the no of constants is more than 255, this will overflow
        (self.constants.values().len() - 1) as u8
    }
}
