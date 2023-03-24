use crate::Value;

/// Registers will store field element values during the
/// duration of the execution of the bytecode.
pub struct Registers {
    inner: Vec<Value>,
}

#[derive(Debug, Clone, Copy)]
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
    pub fn get(&self, index: RegisterIndex) -> Value {
        self.inner[index.inner()]
    }
    /// Sets the value at register with address `index` to `value`
    pub fn set(&mut self, index: RegisterIndex, value: Value) {
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
