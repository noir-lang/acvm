use core::fmt;

use crate::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registers {
    // Registers are a vector of values that might be None.
    // None is used to mark uninitialized registers so we can catch potential mistakes.
    // The reasoning is that instead of returning 0, it is much more likely that such an access is a mistake
    // (e.g. didn't emit the set operation).
    // We grow the register as registers past the end are set, extending with None's.
    pub inner: Vec<Option<Value>>,
}

/// Aims to match a reasonable max register count for a SNARK prover.
/// As well, catches obvious erroneous use of registers.
/// This can be revisited if it proves not enough.
const MAX_REGISTERS: usize = 2_usize.pow(16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterIndex(usize);

/// `RegisterIndex` refers to the index in VM register space.
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

/// Registers will store field element values during the
/// duration of the execution of the bytecode.
impl Registers {
    /// Create a Registers object initialized with definite values
    pub fn load(values: Vec<Value>) -> Registers {
        let inner = values.into_iter().map(Some).collect();
        Self { inner }
    }

    /// Gets the values at register with address `index`
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

    /// Sets the value at register with address `index` to `value`
    pub fn set(&mut self, RegisterIndex(index): RegisterIndex, value: Value) {
        assert!(index < MAX_REGISTERS, "Writing register past maximum!");
        // if size isn't at least index + 1, resize
        let new_register_size = std::cmp::max(index + 1, self.inner.len());
        self.inner.resize(new_register_size, None);
        self.inner[index] = Some(value)
    }
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Registers {{")?;
        for (index, value) in self.inner.iter().enumerate() {
            match value {
                Some(v) => write!(f, "[{}] = {}, ", index, v.to_usize())?,
                None => write!(f, "[{}] = null, ", index)?,
            }
        }
        write!(f, "}}")?;
        Ok(())
    }
}
