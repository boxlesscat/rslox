use std::{fmt, ops};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(b)    => write!(f, "{b}"),
            Self::Nil               => write!(f, "nil"),
            Self::Number(n)   => write!(f, "{n}"),
        }   
    }
}

impl ops::Add for Value {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => Self::Number(l + r),
            (l, r) => panic!("could not add {l:?} and {r:?}"),
        }
    }
}

impl ops::Div for Value {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => Self::Number(l / r),
            (l, r) => panic!("could not add {l:?} and {r:?}"),
        }
    }
}
impl ops::Mul for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => Self::Number(l * r),
            (l, r) => panic!("could not add {l:?} and {r:?}"),
        }
    }
}
impl ops::Sub for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => Self::Number(l - r),
            (l, r) => panic!("could not add {l:?} and {r:?}"),
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