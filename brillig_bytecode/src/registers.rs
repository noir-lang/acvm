use crate::{opcodes::RegisterMemIndex, Typ, Value};
use acir_field::FieldElement;
use serde::{Deserialize, Serialize};

/// Registers will store field element values during the
/// duration of the execution of the bytecode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registers {
    pub inner: Vec<Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterIndex(pub usize);

impl RegisterIndex {
    pub fn inner(self) -> usize {
        self.0
    }
}

impl Registers {
    /// Contiguously load the register with `values`
    pub fn load(values: Vec<Value>) -> Registers {
        Self { inner: values }
    }
    /// Gets the values at register with address `index`
    pub fn get(&self, index: RegisterMemIndex) -> Value {
        match index {
            RegisterMemIndex::Register(register) => self.inner[register.inner()],
            RegisterMemIndex::Constant(constant) => Value { typ: Typ::Field, inner: constant },
            RegisterMemIndex::Memory(_) => todo!("we will implement memory later"),
        }
    }
    /// Sets the value at register with address `index` to `value`
    pub fn set(&mut self, index: RegisterIndex, value: Value) {
        if index.inner() >= self.inner.len() {
            let diff = index.inner() - self.inner.len() + 1;
            self.inner
                .extend(vec![Value { typ: Typ::Field, inner: FieldElement::from(0u128) }; diff])
        }
        self.inner[index.inner()] = value
    }
    /// Returns all of the values in the register
    /// This should be done at the end of the VM
    /// run and will be useful for mapping the values
    /// to witness indices
    pub fn values(self) -> Vec<Value> {
        self.inner
    }
}
