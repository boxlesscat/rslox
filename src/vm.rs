use crate::{chunk::Chunk, value::Value};

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

impl<'a> VM<'a> {
    pub fn new(chunk: &'a Chunk) -> Self {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn intepret(&mut self) -> InterpretResult {
        self.run()
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn run(&mut self) -> InterpretResult {
        use crate::chunk::OpCode::*;
        use InterpretResult::*;
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("          ");
                for slot in &self.stack {
                    print!("[ {} ]", slot);
                }
                println!();
                let disassembler = crate::debug::Disassembler::new(&self.chunk);
                disassembler.disassemble_inst(self.ip);
            }
            let instruction = self.chunk.code()[self.ip];
            self.ip += 1;
            match instruction {
                OpAdd => {
                    let (a, b) = self.binary_op();
                    self.push(a + b);
                }
                OpConstant(constant_index) => {
                    self.push(self.chunk.constants().values()[constant_index as usize]);
                }
                OpDivide => {
                    let (a, b) = self.binary_op();
                    self.push(a / b);
                }
                OpMultiply => {
                    let (a, b) = self.binary_op();
                    self.push(a * b);
                }
                OpNegate => {
                    let res = self.pop();
                    self.push(-res);
                }
                OpReturn => {
                    println!("{}", self.pop());
                    return InterpretOk;
                }
                OpSubtract => {
                    let (a, b) = self.binary_op();
                    self.push(a - b);
                }
            }
        }
    }

    fn binary_op(&mut self) -> (Value, Value) {
        let b = self.pop();
        let a = self.pop();
        (a, b)
    }
}
