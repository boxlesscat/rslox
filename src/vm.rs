use crate::chunk::Chunk;
use crate::compiler::Parser;
use crate::value::Value;

use std::collections::HashMap;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
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
            globals: HashMap::new(),
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

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance] 
    }

    fn reset_stack(&mut self) {
        self.stack = vec![];
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
                let mut disassembler = crate::debug::Disassembler::new(&mut self.chunk);
                disassembler.disassemble_inst(self.ip);
            }
            let instruction = self.chunk.code()[self.ip];
            self.ip += 1;
            macro_rules! bin_op {
                ($op: tt) => {
                    if let (Value::Number(_), Value::Number(_)) = (self.peek(0), self.peek(1)) {
                        let b = self.pop();
                        let a = self.pop();
                        self.push(a $op b);
                    } else {
                        self.runtime_error("Operands must be numbers");
                        return RuntimeError;
                    }
                };
            }
            macro_rules! cmp_op {
                ($op: tt) => {
                    if let (Value::Number(_), Value::Number(_)) = (self.peek(0), self.peek(1)) {
                        let b = self.pop();
                        let a = self.pop();
                        self.push(Value::Bool(a $op b));
                    } else if let (Value::String(_), Value::String(_)) = (self.peek(0), self.peek(1)) {
                        let b = self.pop();
                        let a = self.pop();
                        self.push(Value::Bool(a $op b));
                    } else {
                        self.runtime_error("Operands must be numbers");
                        return RuntimeError;
                    }
                };
            }
            match instruction {
                
                Subtract    => bin_op!(-),
                Multiply    => bin_op!(*),
                Divide      => bin_op!(/),
                Add         =>  {
                    if let (Value::String(_), Value::String(_)) = (self.peek(0), self.peek(1)) {
                        let b = self.pop();
                        let a = self.pop();
                        self.push(a + b);
                    } else {
                        bin_op!(+)
                    }
                }
                
                True        => self.push(Value::Bool(true)),
                False       => self.push(Value::Bool(false)),
                Nil         => self.push(Value::Nil),
                
                Greater     => cmp_op!(>),
                Less        => cmp_op!(<),
                Equal       => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Bool(a == b));
                }

                Constant(constant_index) => {
                    self.push(self.chunk.constants()[constant_index as usize].clone());
                }
                Negate => {
                    if let Value::Number(_) = self.peek(0) {
                        let value = self.pop();
                        self.push(-value);
                    } else {
                        self.runtime_error("Operand must be a number");
                        return RuntimeError
                    }
                }
                Not => {
                    let value = self.pop();
                    let value = self.is_falsey(value);
                    self.push(Value::Bool(value));
                },

                DefineGlobal(global) => {
                    if let Value::String(name) = self.chunk.constants()[global as usize].clone() {
                        self.globals.insert(name, self.peek(0).clone());
                        self.pop();
                    }
                }
                GetGlobal(arg) => {
                    if let Value::String(name) = self.chunk.constants()[arg as usize].clone() {
                        let value = self.globals.get(&name);
                        match value {
                            Some(v) => self.push(v.clone()),
                            None => {
                                self.runtime_error(&format!("Undefined variable '{name}'"));
                                return RuntimeError;
                            }
                        }
                    }
                }
                SetGlobal(arg) => {
                    if let Value::String(name) = self.chunk.constants()[arg as usize].clone() {
                        let val = self.peek(0).clone();
                        let value = self.globals.get_mut(&name);
                        match value {
                            Some(v) => *v = val,
                            None    => {
                                self.runtime_error(&format!("Undefined variable '{name}'"));
                                return RuntimeError;
                            }
                        }
                    }
                }

                GetLocal(slot)  => self.push(self.stack[slot as usize].clone()),
                SetLocal(slot)  => self.stack[slot as usize] = self.peek(0).clone(),

                Jump(offset)        => {
                    self.ip += offset as usize;
                }
                JumpIfFalse(offset) => {
                    if self.is_falsey(self.peek(0).clone()) {
                        self.ip += offset as usize;
                    }
                }
                Loop(offset)        => self.ip -= offset as usize, 

                Return => return Ok,
                Pop => {self.pop();},
                Print => println!("{}", self.pop()),
            }
        }
    }

    fn is_falsey(&self, value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(bool)   => !bool,
            Value::Number(n)    =>  n == 0.0,
            Value::String(s)    =>  s.len() == 0,
        }
    }

    fn runtime_error(&mut self, message: &str) {
        let line = self.chunk.lines()[self.ip - 1];
        eprintln!("{message}\n [line {line} in script]");
        self.reset_stack();
    }

}
