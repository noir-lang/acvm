use crate::native_types::Expression;
use acir_field::FieldElement;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Hash, Copy, Default)]
pub struct BlockId(pub u32);

/// Operation on a block
/// We can either write or read at a block index
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct MemOp {
    /// Can be 0 (read) or 1 (write)
    pub operation: Expression,
    pub index: Expression,
    pub value: Expression,
}

/// Represents operations on a block of length len of data
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryBlock {
    /// Id of the block
    pub id: BlockId,
    /// Length of the memory block
    pub len: u32,
    /// Trace of memory operations
    pub trace: Vec<MemOp>,
}

impl MemoryBlock {
    /// Returns the initialization vector of the MemoryBlock
    pub fn init_phase(&self) -> Vec<Expression> {
        let mut init = Vec::new();
        for i in 0..self.len as usize {
            assert_eq!(
                self.trace[i].operation,
                Expression::one(),
                "Block initialization requires a write"
            );
            let index = self.trace[i]
                .index
                .to_const()
                .expect("Non-const index during Block initialization");
            if index != FieldElement::from(i as i128) {
                todo!(
                    "invalid index when initializing a block, we could try to sort the init phase"
                );
            }
            let value = self.trace[i].value.clone();
            assert!(value.is_degree_one_univariate(), "Block initialization requires a witness");
            init.push(value);
        }
        init
    }
}
