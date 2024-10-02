use crate::chunk::OpCode;
use crate::compiler::Parser;
use crate::value::{self, Function, Native, NativeFunction, Upvalue, Value};
use crate::native::clock::clock;
use crate::native::sqrt::sqrt;

use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;


pub struct CallFrame {
    first_slot: usize,
    ip:         usize,
    function:   Rc<Function>,
    closure:    Rc<RefCell<value::Closure>>,
}

pub struct VM {
    frames:         Vec<CallFrame>,
    stack:          Vec<Value>,
    open_upvalues:  Vec<Rc<RefCell<Upvalue>>>,
    globals:        HashMap<String, Value>,
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            frames:         Vec::new(),
            stack:          Vec::new(), 
            open_upvalues:  Vec::new(),
            globals:        HashMap::new(),
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
        self.push(Value::Function(Rc::clone(&function)));
        let closure = Rc::new(RefCell::new(value::Closure::new(function)));
        self.pop();
        self.push(Value::Closure(Rc::clone(&closure)));
        self.call(closure, 0);
        self.run()
    }

    fn frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().unwrap()
    }

    fn closure(&self) -> Ref<value::Closure> {
        self.frame().closure.borrow()
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
        self.stack.clear();
        self.open_upvalues.clear();
        self.frames.clear();
    }

    fn read_byte(&mut self) -> OpCode {
        self.frame_mut().ip += 1;
        self.closure().function.chunk.code[self.frame().ip - 1].into()
    }

    fn read_constant(&mut self) -> Value {
        let constant = self.read_byte() as usize;
        self.closure().function.chunk.constants[constant].clone()
    }

    fn read_short(&mut self) -> u16 {
        self.frame_mut().ip += 2;
        ((self.closure().function.chunk.code[self.frame().ip - 2] as u16) << 8) | 
        ((self.closure().function.chunk.code[self.frame().ip - 1] as u16))
    }

    fn run(&mut self) -> InterpretResult {
        use crate::chunk::OpCode::*;
        use InterpretResult::*;
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("          ");
                for value in self.stack.iter() {
                    print!("[ {value} ]");
                }
                println!();
                let closure = self.closure();
                let disassembler = crate::debug::Disassembler::new(&closure.function.chunk);
                disassembler.disassemble_instruction(self.frame().ip);
            }
            let instruction = self.frame().function.chunk.code[self.frame().ip].into();
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
                    if let Value::String(a) = self.peek(1).clone() {
                        let b = self.peek(0).clone();
                        let b = b.to_string();
                        self.pop();
                        self.pop();
                        self.push(Value::String(Rc::new(String::with_capacity(a.len() + b.len()) + &a + &b)));
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

                Constant => {
                    let constant = self.read_constant();
                    self.push(constant);
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

                DefineGlobal => {
                    if let Value::String(name) = self.read_constant() {
                        let value = self.pop();
                        self.globals.insert(name.to_string(), value);
                    }
                }
                GetGlobal => {
                    if let Value::String(name) = self.read_constant() {
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
                SetGlobal => {
                    if let Value::String(name) = self.read_constant() {
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

                GetLocal        => {
                    let slot = self.read_byte();
                    self.push(self.stack[self.frame().first_slot + slot as usize].clone())
                }
                SetLocal        => {
                    let mut slot = self.read_byte() as usize;
                    slot += self.frame_mut().first_slot;
                    self.stack[slot] = self.peek(0).clone();
                }

                Jump            => {
                    let offset = self.read_short();
                    self.frame_mut().ip += offset as usize;
                }
                JumpIfFalse     => {
                    let offset = self.read_short();
                    if self.is_falsey(self.peek(0).clone()) {
                        self.frame_mut().ip += offset as usize;
                    }
                }
                Loop            => {
                    let offset = self.read_short();
                    self.frame_mut().ip -= offset as usize
                }, 
                Call            => {
                    let arg_count = self.read_byte() as u8;
                    if !self.call_value(self.peek(arg_count as usize).clone(), arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
                Closure => {
                    let function = self.read_constant();
                    if let Value::Function(function) = function {
                        let upvalue_count = function.upvalue_count;
                        let closure = value::Closure::new(function);
                        let closure = RefCell::new(closure);
                        let closure = Rc::new(closure);
                        self.push(Value::Closure(Rc::clone(&closure)));
                        for _ in 0..upvalue_count {
                            let is_local = self.read_byte() as u8;
                            let index = self.read_byte() as usize;
                            if is_local != 0 {
                                closure.borrow_mut().upvalues.push(self.capture_upvalue(self.frame().first_slot + index));
                            } else {
                                closure.borrow_mut().upvalues.push(self.closure().upvalues[index].clone());
                            }
                        }
                    };
                }
                GetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let upvalue = self.frame().closure.borrow().upvalues[slot].clone();
                    if upvalue.borrow().closed.is_some() {
                        self.push(Value::Upvalue(upvalue));
                    } else {
                        if let Some(closed) = upvalue.borrow().closed.clone() {
                            self.push(closed);
                        } else {
                            self.push(self.stack[upvalue.borrow().location].clone());
                        }
                    }
                }
                SetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let location = self.closure().upvalues[slot].borrow().location;
                    let value = self.peek(0).clone();
                    if self.closure().upvalues[slot].borrow().closed.is_some() {
                        self.closure().upvalues[slot].borrow_mut().closed = Some(value);                        
                    } else {
                        self.stack[location] = value;
                    }
                }
                CloseUpvalue => {
                    self.close_upvalues(self.stack.len() - 1);
                    self.pop();
                }
                Return => {
                    let result = self.pop();
                    let slot = self.frame().first_slot;
                    self.close_upvalues(slot);
                    self.frames.pop();
                    if self.frames.is_empty() {
                        self.pop();
                        return InterpretResult::Ok;
                    }
                    self.stack.truncate(slot);
                    self.push(result);
                },
                Pop => {
                    self.pop();
                },
                Print => println!("{}", self.pop()),
            }
        }
    }

    fn capture_upvalue(&mut self, local: usize) -> Rc<RefCell<Upvalue>> {
        for upvalue in &self.open_upvalues {
            if upvalue.borrow().location == local {
                return Rc::clone(&upvalue);
            }
        }
        let created_upvalue = Upvalue::new(local);
        let created_upvalue = Rc::new(RefCell::new(created_upvalue));
        self.open_upvalues.push(Rc::clone(&created_upvalue));
        created_upvalue
    }
    
    fn close_upvalues(&mut self, last: usize) {
        let mut i = 0;
        while i < self.open_upvalues.len() {
            let upvalue = &mut self.open_upvalues[i];
            if upvalue.borrow().location >= last {
                let value = Some(self.stack[upvalue.borrow().location].clone());
                upvalue.borrow_mut().closed = value;
                let last_upvalue = self.open_upvalues.pop();
                if i != self.open_upvalues.len() {
                    self.open_upvalues[i] = last_upvalue.unwrap();
                }
            } else {
                i += 1;
            }
        }
    }

    fn call_value(&mut self, callee: Value, arg_count: u8) -> bool {
        match callee {
            Value::Closure(closure)   => self.call(closure, arg_count),
            Value::Native(native)       => {
                let res = (native.function)(arg_count, &self.stack[self.stack.len() - arg_count as usize..self.stack.len()]);
                let len = self.stack.len() - arg_count as usize - 1;
                self.stack.truncate(len);
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

    fn call(&mut self, closure: Rc<RefCell<value::Closure>>, arg_count: u8) -> bool {
        if arg_count as usize != closure.borrow().function.arity {
            self.runtime_error(&format!("Expected {} arguments but got {}", closure.borrow().function.arity, arg_count));
            return false;
        }
        let function = Rc::clone(&closure.borrow().function);
        self.frames.push(CallFrame {
            function,
            ip: 0,
            first_slot: self.stack.len() - arg_count as usize - 1,
            closure,
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
            let function = &frame.closure.borrow().function;
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
