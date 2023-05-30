use crate::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registers {
    // Keep track of registers as an option to represent uninitialized registers
    // These occur when we set a register value at an uncontiguous index, past currently defined registers
    // This could just store 0's, but it would be
    pub inner: Vec<Option<Value>>,
}

const MAX_REGISTERS: usize = 2_usize.pow(16);

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
    pub fn load(values: Vec<Value>) -> Registers {
        let inner = values.into_iter().map(Some).collect();
        Self { inner }
    }

    pub fn get(&self, register_index: RegisterIndex) -> Value {
        let index = register_index.to_usize();
        assert!(index < MAX_REGISTERS, "Reading register past maximum!");
        assert!(
            index < self.inner.len(),
            "Reading uninitialized register {} (current max index {})!",
            index,
            self.inner.len() - 1
        );
        self.inner[index].expect("Reading uninitialized register!")
    }

    pub fn set(&mut self, RegisterIndex(index): RegisterIndex, value: Value) {
        assert!(index < MAX_REGISTERS, "Writing register past maximum!");
        let new_register_size = std::cmp::max(index + 1, self.inner.len());
        self.inner.resize(new_register_size, None);
        self.inner[index] = Some(value)
    }

    pub fn values(self) -> Vec<Option<Value>> {
        self.inner
    }
}
