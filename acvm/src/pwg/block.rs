use std::collections::HashMap;

use acir::{
    circuit::opcodes::MemOp,
    native_types::{Witness, WitnessMap},
    FieldElement,
};

use super::OpcodeResolutionError;
use super::{arithmetic::ArithmeticSolver, get_value, insert_value, witness_to_value};

type MemoryIndex = u32;

/// Maintains the state for solving Block opcode
/// block_value is the value of the Block at the solved_operations step
/// solved_operations is the number of solved elements in the block
#[derive(Default)]
pub(super) struct BlockSolver {
    block_value: HashMap<MemoryIndex, FieldElement>,
}

impl BlockSolver {
    fn write_memory_index(&mut self, index: MemoryIndex, value: FieldElement) {
        self.block_value.insert(index, value);
    }

    fn read_memory_index(&self, index: MemoryIndex) -> FieldElement {
        self.block_value.get(&index).copied().expect("Should not read uninitialized memory")
    }

    /// Set the block_value from a MemoryInit opcode
    pub(crate) fn init(
        &mut self,
        init: &[Witness],
        initial_witness: &WitnessMap,
    ) -> Result<(), OpcodeResolutionError> {
        for (memory_index, witness) in init.iter().enumerate() {
            self.write_memory_index(
                memory_index as MemoryIndex,
                *witness_to_value(initial_witness, *witness)?,
            );
        }
        Ok(())
    }

    pub(crate) fn solve_memory_op(
        &mut self,
        op: &MemOp,
        initial_witness: &mut WitnessMap,
    ) -> Result<(), OpcodeResolutionError> {
        let operation = get_value(&op.operation, initial_witness)?;

        // Find the memory index associated with this memory operation.
        let index = get_value(&op.index, initial_witness)?;
        let memory_index = index.try_to_u64().unwrap() as MemoryIndex;

        // Calculate the value associated with this memory operation.
        //
        // In read operations, this corresponds to the witness index at which the value from memory will be written.
        // In write operations, this corresponds to the expression which will be written to memory.
        let value = ArithmeticSolver::evaluate(&op.value, initial_witness);

        // `operation == 0` implies a read operation. (`operation == 1` implies write operation).
        let is_read_operation = operation.is_zero();

        if is_read_operation {
            // `value_read = arr[memory_index]`
            //
            // This is the value that we want to read into; i.e. copy from the memory block
            // into this value.
            let value_read_witness = value.to_witness().expect(
                "Memory must be read into a specified witness index, encountered an Expression",
            );

            let value_in_array = self.read_memory_index(memory_index);

            insert_value(&value_read_witness, value_in_array, initial_witness)
        } else {
            // `arr[memory_index] = value_write`
            //
            // This is the value that we want to write into; i.e. copy from `value_write`
            // into the memory block.
            let value_write = value;

            let value_to_write = get_value(&value_write, initial_witness)?;

            self.write_memory_index(memory_index, value_to_write);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acir::{
        circuit::opcodes::MemOp,
        native_types::{Witness, WitnessMap},
        FieldElement,
    };

    use super::BlockSolver;

    #[test]
    fn test_solver() {
        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(1u128)),
            (Witness(3), FieldElement::from(2u128)),
        ]));

        let init = vec![Witness(1), Witness(2)];

        let trace = vec![
            MemOp::write_to_mem_index(FieldElement::from(2u128).into(), Witness(3).into()),
            MemOp::read_at_mem_index(FieldElement::one().into(), Witness(4)),
        ];

        let mut block_solver = BlockSolver::default();
        block_solver.init(&init, &initial_witness).unwrap();

        for op in trace {
            block_solver.solve_memory_op(&op, &mut initial_witness).unwrap();
        }
        assert_eq!(initial_witness[&Witness(4)], FieldElement::one());
    }
}
