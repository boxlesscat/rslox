use crate::chunk::Chunk;

use std::fmt;
use std::ops;
use std::rc::Rc;


#[derive(Debug, Clone)]
pub struct Closure {
    pub function: Rc<Function>,
}

#[derive(Debug, Default, Clone)]
pub struct Function {
    pub arity:  usize,
    pub name:   String,
    pub chunk:  Chunk
}

pub type Native = fn(u8, &[Value]) -> Result<Value, &str>;

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub arity:      u8,
    pub name:       Rc<String>,
    pub function:   Native,
}

#[derive(Clone, Debug, Default)]
pub enum Value {
    Bool(bool),
    Closure(Rc<Closure>),
    #[default]
    Nil,
    Number(f64),
    String(Rc<String>),
    Function(Rc<Function>),
    Native(Rc<NativeFunction>),
}

impl Closure {
    pub fn new(function: Rc<Function>) -> Self {
        Self {
            function
        }
    }
}

impl Function {
    pub fn new() -> Self {
        Self {
            arity: 0,
            name: String::new(),
            chunk: Chunk::default(),
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
            Self::Closure(c)   => if c.function.name.len() == 0 {
                write!(f, "<script>")
            } else {
                write!(f, "<fn {}>", c.function.name)
            }
            Self::Native(n)     => write!(f, "<fn {}>", n.name),
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