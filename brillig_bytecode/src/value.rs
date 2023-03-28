use acir_field::FieldElement;
use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Types of values allowed in the VM
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Typ {
    Field,
    Unsigned { bit_size: u32 },
    Signed { bit_size: u32 },
}

/// Value represents a Value in the VM
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {
    pub typ: Typ,
    pub inner: FieldElement,
}

impl Value {
    /// Returns true if the Value represents `zero`
    pub fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }
    /// Performs the multiplicative inverse of `Value`
    pub fn inverse(&self) -> Value {
        let value = match self.typ {
            Typ::Field => self.inner.inverse(),
            Typ::Unsigned { bit_size } => {
                todo!("TODO")
            }
            Typ::Signed { bit_size } => todo!("TODO"),
        };
        Value { typ: self.typ, inner: value }
    }
}

impl From<u128> for Value {
    fn from(value: u128) -> Self {
        Value { typ: Typ::Field, inner: FieldElement::from(value) }
    }
}

impl From<FieldElement> for Value {
    fn from(value: FieldElement) -> Self {
        Value { typ: Typ::Field, inner: value }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        if value {
            Value { typ: Typ::Unsigned { bit_size: 1 }, inner: FieldElement::one() }
        } else {
            Value { typ: Typ::Unsigned { bit_size: 1 }, inner: FieldElement::zero() }
        }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        Value { typ: self.typ, inner: self.inner + rhs.inner }
    }
}
impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        Value { typ: self.typ, inner: self.inner - rhs.inner }
    }
}
impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        Value { typ: self.typ, inner: self.inner * rhs.inner }
    }
}
impl Div for Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        Value { typ: self.typ, inner: self.inner / rhs.inner }
    }
}
impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        Value { typ: self.typ, inner: -self.inner }
    }
}
