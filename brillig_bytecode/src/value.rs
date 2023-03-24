use acir::FieldElement;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Values that we can store in registers
pub enum Value {
    Field(FieldElement),
}

impl Value {
    /// Returns true if the Value represents `zero`
    pub fn is_zero(&self) -> bool {
        match self {
            Value::Field(val) => *val == FieldElement::zero(),
        }
    }
    /// Performs the multiplicative inverse of `Value`
    pub fn inverse(&self) -> Value {
        match self {
            Value::Field(val) => Value::Field(val.inverse()),
        }
    }
}

impl From<u128> for Value {
    fn from(value: u128) -> Self {
        Value::Field(FieldElement::from(value))
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Field(lhs), Value::Field(rhs)) => Value::Field(lhs + rhs),
        }
    }
}
impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Field(lhs), Value::Field(rhs)) => Value::Field(lhs - rhs),
        }
    }
}
impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Field(lhs), Value::Field(rhs)) => Value::Field(lhs * rhs),
        }
    }
}
impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Field(lhs), Value::Field(rhs)) => Value::Field(lhs / rhs),
        }
    }
}
impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            Value::Field(input) => Value::Field(-input),
        }
    }
}
