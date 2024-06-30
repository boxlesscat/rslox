use crate::compiler::Parser;
use crate::value::{Function, Native, NativeFunction, Value};
use crate::native::clock::clock;
use crate::native::sqrt::sqrt;

use std::collections::HashMap;
use std::mem;
use std::rc::Rc;


const U8_COUNT:     usize = u8::MAX as usize + 1;
const FRAMES_MAX:   usize = 32;
const STACK_MAX:    usize = FRAMES_MAX * U8_COUNT;

pub struct CallFrame {
    first_slot: usize,
    function:   Rc<Function>,
    ip:         usize,
}

pub struct VM {
    frames:         Vec<CallFrame>,
    globals:        HashMap<String, Value>,
    stack:          [Value; STACK_MAX],
    stack_top:      usize,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            frames: Vec::new(),
            globals: HashMap::new(),
            stack: unsafe {
                mem::zeroed() // mem::MaybeUninit::uninit().assume_init() gives seg fault with the build release executable but not dev executable
            },
            stack_top: 0,
        };
        vm.define_native(0, "clock", clock);
        vm.define_native(1, "sqrt", sqrt);
        vm
    }

    pub fn interpret(&mut self, source: &str) -> InterpretResult {
        let mut compiler = Parser::new(source);
        let result = compiler.compile();
        if result.is_none() {
            return InterpretResult::CompileError;
        }
        let function = result.unwrap();
        let function = Rc::new(function);
        self.push(Value::Function(Rc::clone(&function)));
        self.call(function, 0);
        self.run()
    }

    fn frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        mem::take(&mut self.stack[self.stack_top])
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack_top - 1 - distance] 
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
        self.frames.clear();
    }

    fn run(&mut self) -> InterpretResult {
        use crate::chunk::OpCode::*;
        use InterpretResult::*;
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("          ");
                for slot in 0..self.stack_top {
                    print!("[ {} ]", self.stack[slot]);
                }
                println!();
                let disassembler = crate::debug::Disassembler::new(&self.frame().function.chunk);
                disassembler.disassemble_inst(self.frame().ip); //?
            }
            let instruction = self.frame().function.chunk.code[self.frame().ip];
            self.frame_mut().ip += 1;


            macro_rules! bin_op {
                ($op: tt) => {
                    if let (Value::Number(b), Value::Number(a)) = (self.peek(0).clone(), self.peek(1).clone()) {
                        self.pop();
                        self.pop();
                        self.push(Value::Number(a $op b));
                    } else {
                        self.runtime_error("Operands must be numbers");
                        return RuntimeError;
                    }
                };
            }
            macro_rules! cmp_op {
                ($op: tt) => {
                    if let (Value::Number(b), Value::Number(a)) = (self.peek(0).clone(), self.peek(1).clone()) {
                        self.pop();
                        self.pop();
                        self.push(Value::Bool(a $op b));
                    } else if let (Value::String(b), Value::String(a)) = (self.peek(0).clone(), self.peek(1).clone()) {
                        self.pop();
                        self.pop();
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
                    if let (Value::String(b), Value::String(a)) = (self.peek(0).clone(), self.peek(1).clone()) {
                        self.pop();
                        self.pop();
                        self.push(Value::String(Rc::new(String::with_capacity(b.len() + a.len()) + &b + &a)));
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

                Constant(constant_index) => self.push(self.frame().function.chunk.constants[constant_index as usize].clone()),
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
                    if let Value::String(name) = self.frame_mut().function.chunk.constants[global as usize].clone() {
                        let value = self.pop();
                        self.globals.insert(name.to_string(), value);
                    }
                }
                GetGlobal(arg) => {
                    if let Value::String(name) = self.frame_mut().function.chunk.constants[arg as usize].clone() {
                        let value = self.globals.get(&name.to_string());
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
                    if let Value::String(name) = self.frame_mut().function.chunk.constants[arg as usize].clone() {
                        let val = self.peek(0).clone();
                        let value = self.globals.get_mut(&name.to_string());
                        match value {
                            Some(v) => *v = val,
                            None    => {
                                self.runtime_error(&format!("Undefined variable '{name}'"));
                                return RuntimeError;
                            }
                        }
                    }
                }

                GetLocal(slot)  => self.push(self.stack[self.frame().first_slot + slot as usize].clone()),
                SetLocal(slot)  => self.stack[self.frame_mut().first_slot + slot as usize] = self.peek(0).clone(),

                Jump(offset)        => {
                    self.frame_mut().ip += offset as usize;
                }
                JumpIfFalse(offset) => {
                    if self.is_falsey(self.peek(0).clone()) {
                        self.frame_mut().ip += offset as usize;
                    }
                }
                Loop(offset)        => self.frame_mut().ip -= offset as usize, 
                
                Call(arg_count)     => {
                    if !self.call_value(self.peek(arg_count as usize).clone(), arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
                Return => {
                    let result = self.pop();
                    let slot = self.frame().first_slot;
                    self.frames.pop();
                    if self.frames.is_empty() {
                        self.pop();
                        return InterpretResult::Ok;
                    }
                    self.stack_top = slot;
                    self.push(result);
                },
                Pop => {
                    self.pop();
                },
                Print => println!("{}", self.pop()),
            }
        }
    }

    fn call_value(&mut self, callee: Value, arg_count: u8) -> bool {
        match callee {
            Value::Function(function)   => self.call(function, arg_count),
            Value::Native(native)       => {
                let res = (native.function)(arg_count, &self.stack[self.stack_top - arg_count as usize..self.stack_top]);
                self.stack_top -= arg_count as usize + 1;
                match res {
                    Err(message) => {
                        self.runtime_error(&message.to_string());
                        false
                    }
                    Ok(value)   => {
                        self.push(value);
                        true
                    }
                }
            }
            _ => {
                self.runtime_error("Can only call functions and classes.");
                false            
            }
        }
    }

    fn define_native(&mut self, arity: u8, name: &str, native: Native) {
        let function = NativeFunction {
            arity,
            function: native,
            name: Rc::new(name.to_string()),
        };
        self.globals.insert(name.to_string(), Value::Native(Rc::new(function)));
    }

    fn call(&mut self, function: Rc<Function>, arg_count: u8) -> bool {
        if arg_count as usize != function.arity {
            self.runtime_error(&format!("Expected {} arguments but got {}", function.arity, arg_count));
            return false;
        }
        let stack_top = self.stack_top;
        self.frames.push(CallFrame {
            function,
            ip: 0,
            first_slot: stack_top - arg_count as usize - 1,
        });
        true
    }

    fn is_falsey(&self, value: Value) -> bool {
        match value {
            Value::Nil => true,
            Value::Bool(bool)   => !bool,
            Value::Number(n)    =>  n == 0.0,
            Value::String(s)    =>  s.len() == 0,
            _  =>  false,
        }
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{message}");
        for frame in self.frames.iter().rev() {
            let function = &frame.function;
            eprint!("[line {}] in ", function.chunk.lines[frame.ip - 1]);
            if function.name.is_empty() {
                eprintln!("script");
            } else {
                eprintln!("{}", function.name);
            }
        }
        self.reset_stack();
    }

}
