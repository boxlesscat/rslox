use std::fmt;
use std::ops;
use std::rc::Rc;


#[derive(Clone, Debug, Default)]
pub enum Value {
    Bool(bool),
    #[default]
    Nil,
    Number(f64),
    String(Rc<String>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(b)   => write!(f, "{b}"),
            Self::Nil       => write!(f, "nil"),
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
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