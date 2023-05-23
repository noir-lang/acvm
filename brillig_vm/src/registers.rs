use crate::Value;
use serde::{Deserialize, Serialize};

/// Registers will store field element values during the
/// duration of the execution of the bytecode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registers {
    pub inner: Vec<Value>,
}

impl IntoIterator for Registers {
    type Item = Value;
    type IntoIter = RegistersIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        RegistersIntoIterator { registers: self, index: 0 }
    }
}
pub struct RegistersIntoIterator {
    registers: Registers,
    index: usize,
}

impl Iterator for RegistersIntoIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.registers.inner.len() {
            return None;
        }

        self.index += 1;
        Some(self.registers.inner[self.index - 1])
    }
}
/// `RegisterIndex` refers to the index of a register in the VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterIndex(usize);

impl RegisterIndex {
    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for RegisterIndex {
    fn from(value: usize) -> Self {
        RegisterIndex(value)
    }
}

impl Registers {
    /// Contiguously load the register with `values`
    pub fn load(values: Vec<Value>) -> Registers {
        Self { inner: values }
    }

    /// Gets the values at register with address `index`
    pub fn get(&self, register: RegisterIndex) -> Value {
        self.inner[register.to_usize()]
    }

    /// Sets the value at register with address `index` to `value`
    pub fn set(&mut self, index: RegisterIndex, value: Value) {
        if index.to_usize() >= self.inner.len() {
            let diff = index.to_usize() - self.inner.len() + 1;
            self.inner.extend(vec![Value::from(0u128); diff])
        }
        self.inner[index.to_usize()] = value
    }

    /// Returns all of the values in the register
    /// This should be done at the end of the VM
    /// run and will be useful for mapping the values
    /// to witness indices
    pub fn values(self) -> Vec<Value> {
        self.inner
    }
}
