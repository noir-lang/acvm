use std::collections::HashMap;

use acir::{
    circuit::opcodes::MemOp,
    native_types::{Witness, WitnessMap},
    FieldElement,
};

use super::{
    any_witness_from_expression,
    arithmetic::{ArithmeticSolver, GateStatus},
    get_value, insert_value,
};
use super::{OpcodeNotSolvable, OpcodeResolution, OpcodeResolutionError};

type MemoryIndex = u32;

/// Maintains the state for solving Block opcode
/// block_value is the value of the Block at the solved_operations step
/// solved_operations is the number of solved elements in the block
#[derive(Default)]
pub(super) struct BlockSolver {
    block_value: HashMap<MemoryIndex, FieldElement>,
    solved_operations: usize,
}

impl BlockSolver {
    fn insert_value(&mut self, index: MemoryIndex, value: FieldElement) {
        self.block_value.insert(index, value);
    }

    fn get_value(&self, index: MemoryIndex) -> Option<FieldElement> {
        self.block_value.get(&index).copied()
    }

    /// Set the block_value from a MemoryInit opcode
    pub(crate) fn init(&mut self, init: &[Witness], initial_witness: &WitnessMap) {
        for (i, w) in init.iter().enumerate() {
            self.insert_value(i as u32, initial_witness[w]);
        }
    }

    // Helper function which tries to solve a Block opcode
    // As long as operations are resolved, we update/read from the block_value
    // We stop when an operation cannot be resolved
    fn solve_helper(
        &mut self,
        initial_witness: &mut WitnessMap,
        trace: &[MemOp],
    ) -> Result<(), OpcodeResolutionError> {
        for block_op in trace.iter().skip(self.solved_operations) {
            self.solve_memory_op(block_op, initial_witness)?;
            self.solved_operations += 1;
        }
        Ok(())
    }

    pub(crate) fn solve_memory_op(
        &mut self,
        op: &MemOp,
        initial_witness: &mut WitnessMap,
    ) -> Result<(), OpcodeResolutionError> {
        let missing_assignment = |witness: Option<Witness>| {
            OpcodeResolutionError::OpcodeNotSolvable(OpcodeNotSolvable::MissingAssignment(
                witness.unwrap().0,
            ))
        };

        let op_expr = ArithmeticSolver::evaluate(&op.operation, initial_witness);
        let operation = op_expr
            .to_const()
            .ok_or_else(|| missing_assignment(any_witness_from_expression(&op_expr)))?;

        // Find the memory index associated with this memory operation.
        let index_expr = ArithmeticSolver::evaluate(&op.index, initial_witness);
        let index = index_expr
            .to_const()
            .ok_or_else(|| missing_assignment(any_witness_from_expression(&index_expr)))?;
        let memory_index = index.try_to_u64().unwrap() as MemoryIndex;

        // Calculate the value associated with this memory operation.
        let value = ArithmeticSolver::evaluate(&op.value, initial_witness);

        // `operation == 0` implies a read operation. (`operation == 1` implies write operation).
        let is_read_operation = operation.is_zero();

        if is_read_operation {
            // value_read = arr[memory_index]
            //
            // This is the value that we want to read into; ie copy from the memory block
            // into this value.
            let value_read_witness = value.to_witness().expect("This should be a witness");

            // TODO: change error message
            let value_in_array =
                self.get_value(memory_index).ok_or_else(|| missing_assignment(Some(Witness(0))))?;

            insert_value(&value_read_witness, value_in_array, initial_witness)?;
        } else {
            // arr[memory_index] = value_write
            //
            // This is the value that we want to write into; ie copy from value_write
            // into the memory block.
            let value_write = value;

            let value_to_write = get_value(&value_write, initial_witness).expect("Change");

            self.insert_value(memory_index, value_to_write);
        }
        Ok(())
    }

    // Try to solve block operations from the trace
    // The function calls solve_helper() for solving the opcode
    // and converts its result into GateResolution
    pub(crate) fn solve(
        &mut self,
        initial_witness: &mut WitnessMap,
        trace: &[MemOp],
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        let initial_solved_operations = self.solved_operations;

        match self.solve_helper(initial_witness, trace) {
            Ok(()) => Ok(OpcodeResolution::Solved),
            Err(OpcodeResolutionError::OpcodeNotSolvable(err)) => {
                if self.solved_operations > initial_solved_operations {
                    Ok(OpcodeResolution::InProgress)
                } else {
                    Ok(OpcodeResolution::Stalled(err))
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use acir::{
        circuit::opcodes::MemOp,
        native_types::{Expression, Witness, WitnessMap},
        FieldElement,
    };

    use super::BlockSolver;
    use crate::pwg::insert_value;

    #[test]
    fn test_solver() {
        let mut index = FieldElement::zero();
        let mut trace = vec![MemOp {
            operation: Expression::one(),
            index: Expression::from_field(index),
            value: Expression::from(Witness(1)),
        }];
        index += FieldElement::one();
        trace.push(MemOp {
            operation: Expression::one(),
            index: Expression::from_field(index),
            value: Expression::from(Witness(2)),
        });
        index += FieldElement::one();
        trace.push(MemOp {
            operation: Expression::one(),
            index: Expression::from_field(index),
            value: Expression::from(Witness(3)),
        });
        trace.push(MemOp {
            operation: Expression::zero(),
            index: Expression::one(),
            value: Expression::from(Witness(4)),
        });
        let mut initial_witness = WitnessMap::new();
        let mut value = FieldElement::zero();
        insert_value(&Witness(1), value, &mut initial_witness).unwrap();
        value = FieldElement::one();
        insert_value(&Witness(2), value, &mut initial_witness).unwrap();
        value = value + value;
        insert_value(&Witness(3), value, &mut initial_witness).unwrap();
        let mut block_solver = BlockSolver::default();
        block_solver.solve(&mut initial_witness, &trace).unwrap();
        assert_eq!(initial_witness[&Witness(4)], FieldElement::one());
    }
}
