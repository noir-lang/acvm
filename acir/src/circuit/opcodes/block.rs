use std::io::{Read, Write};

use crate::native_types::Expression;
use crate::serialization::{read_u32, write_u32};
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
    pub fn read<R: Read>(mut reader: R) -> std::io::Result<Self> {
        let id = read_u32(&mut reader)?;
        let len = read_u32(&mut reader)?;
        let trace_len = read_u32(&mut reader)?;
        let mut trace = Vec::with_capacity(len as usize);
        for _i in 0..trace_len {
            let operation = Expression::read(&mut reader)?;
            let index = Expression::read(&mut reader)?;
            let value = Expression::read(&mut reader)?;
            trace.push(MemOp { operation, index, value });
        }
        Ok(MemoryBlock { id: BlockId(id), len, trace })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
        write_u32(&mut writer, self.id.0)?;
        write_u32(&mut writer, self.len)?;
        write_u32(&mut writer, self.trace.len() as u32)?;

        for op in &self.trace {
            op.operation.write(&mut writer)?;
            op.index.write(&mut writer)?;
            op.value.write(&mut writer)?;
        }
        Ok(())
    }

    /// Returns the initialization vector of the MemoryBlock
    pub fn init_phase(&self) -> Vec<Expression> {
        let mut init = Vec::new();
        for i in 0..self.len as usize {
            assert_eq!(
                self.trace[i].operation,
                Expression::one(),
                "Block initialization require a write"
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
