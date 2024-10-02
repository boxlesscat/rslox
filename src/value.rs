use crate::chunk::Chunk;

use std::cell::RefCell;
use std::fmt;
use std::ops;
use std::rc::Rc;


#[derive(Debug, Clone)]
pub struct Closure {
    pub function: Rc<Function>,
    pub upvalues: Vec<Rc<RefCell<Upvalue>>>,
}

#[derive(Debug, Clone, Default)]
pub struct Upvalue {
    pub location: usize,
    pub closed: Option<Value>,
}

#[derive(Debug, Default, Clone)]
pub struct Function {
    pub arity:  usize,
    pub name:   String,
    pub chunk:  Chunk,
    pub upvalue_count: usize,
}

pub type Native = fn(u8, &[Value]) -> Result<Value, String>;

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub arity:      u8,
    pub name:       Rc<String>,
    pub function:   Native,
}

#[derive(Clone, Debug, Default)]
pub enum Value {
    Bool(bool),
    Closure(Rc<RefCell<Closure>>),
    #[default]
    Nil,
    Number(f64),
    String(Rc<String>),
    Function(Rc<Function>),
    Native(Rc<NativeFunction>),
    Upvalue(Rc<RefCell<Upvalue>>),
}

impl Closure {
    pub fn new(function: Rc<Function>) -> Self {
        Self {
            upvalues: Vec::with_capacity(function.upvalue_count),
            function,
        }
    }
}

impl Function {
    pub fn new() -> Self {
        Self {
            arity: 0,
            name: String::new(),
            chunk: Chunk::default(),
            upvalue_count: 0,
        }
    }
}

impl  Upvalue {
    pub fn new(slot: usize) -> Self {
        Self {
            location: slot,
            closed: None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(b)       => write!(f, "{b}"),
            Self::Nil           => write!(f, "nil"),
            Self::Number(n)     => write!(f, "{n}"),
            Self::String(s)     => write!(f, "{s}"),
            Self::Function(n)   => if n.name.len() == 0 {
                write!(f, "<script>")
            } else {
                write!(f, "<fn {}>", n.name)
            }
            Self::Closure(c)   => if c.borrow().function.name.len() == 0 {
                write!(f, "<script>")
            } else {
                write!(f, "<fn {}>", c.borrow().function.name)
            }
            Self::Native(n)     => write!(f, "<fn {}>", n.name),
            Self::Upvalue(u) => match &u.borrow().closed {
                Some(value) => write!(f, "{}", value),
                None => write!(f, "<closed>"),
            },
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(a),     Self::Bool(b))      => a == b,
            (Self::Nil,         Self::Nil)          => true,
            (Self::Number(a),   Self::Number(b))    => a == b,
            (Self::String(a),   Self::String(b))    => a == b,
            _ => false
        }
    }
}

impl ops::Neg for Value {
    type Output = Self;
    fn neg(self) -> Self::Output {
        match self {
            Self::Number(n) => Self::Number(-n),
            n => panic!("could not negate {n:?}"),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<&Rc<RefCell<Closure>>> for Value {
    fn from(value: &Rc<RefCell<Closure>>) -> Self {
        Self::Closure(Rc::clone(&value))
    }
}

impl From<&Rc<Function>> for Value {
    fn from(value: &Rc<Function>) -> Self {
        Self::Function(Rc::clone(&value))
    }
}

impl From<NativeFunction> for Value {
    fn from(value: NativeFunction) -> Self {
        Self::Native(Rc::new(value))
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(Rc::new(value))
    }
}

impl From<Rc<RefCell<Upvalue>>> for Value {
    fn from(value: Rc<RefCell<Upvalue>>) -> Self {
        Self::Upvalue(value)
    }
}