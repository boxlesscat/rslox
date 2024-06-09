use crate::chunk::Chunk;
use crate::compiler::Parser;
use crate::value::Value;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

#[derive(PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl VM {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        self.ip = 0;
        let mut compiler = Parser::new(source);
        let res = compiler.compile();
        self.chunk = match res {
            Ok(chunk) => chunk,
            _ => return InterpretResult::CompileError,
        };
        return self.run();
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
                Add => {
                    let (a, b) = self.binary_op();
                    self.push(a + b);
                }
                Constant(constant_index) => {
                    self.push(self.chunk.constants().values()[constant_index as usize]);
                }
                Divide => {
                    let (a, b) = self.binary_op();
                    self.push(a / b);
                }
                Multiply => {
                    let (a, b) = self.binary_op();
                    self.push(a * b);
                }
                Negate => {
                    let res = self.pop();
                    self.push(-res);
                }
                Return => {
                    println!("{}", self.pop());
                    return Ok;
                }
                Subtract => {
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
