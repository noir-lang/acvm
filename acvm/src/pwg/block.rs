use std::collections::HashMap;

use acir::{
    circuit::opcodes::MemOp,
    native_types::{Witness, WitnessMap},
    FieldElement,
};

use super::{
    arithmetic::{ArithmeticSolver, GateStatus},
    insert_value,
};
use super::{OpcodeNotSolvable, OpcodeResolutionError};

/// Maintains the state for solving Block opcode
/// block_value is the value of the Block at the solved_operations step
/// solved_operations is the number of solved elements in the block
#[derive(Default)]
pub(super) struct BlockSolver {
    block_value: HashMap<u32, FieldElement>,
}

impl BlockSolver {
    fn insert_value(&mut self, index: u32, value: FieldElement) {
        self.block_value.insert(index, value);
    }

    fn get_value(&self, index: u32) -> Option<FieldElement> {
        self.block_value.get(&index).copied()
    }

    /// Set the block_value from a MemoryInit opcode
    pub(crate) fn init(&mut self, init: &[Witness], initial_witness: &WitnessMap) {
        for (i, w) in init.iter().enumerate() {
            self.insert_value(i as u32, initial_witness[w]);
        }
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
        let operation = op_expr.to_const().ok_or_else(|| {
            missing_assignment(ArithmeticSolver::any_witness_from_expression(&op_expr))
        })?;
        let index_expr = ArithmeticSolver::evaluate(&op.index, initial_witness);
        let index = index_expr.to_const().ok_or_else(|| {
            missing_assignment(ArithmeticSolver::any_witness_from_expression(&index_expr))
        })?;
        let index = index.try_to_u64().unwrap() as u32;
        let value = ArithmeticSolver::evaluate(&op.value, initial_witness);
        let value_witness = ArithmeticSolver::any_witness_from_expression(&value);
        if value.is_const() {
            self.insert_value(index, value.q_c);
        } else if operation.is_zero() && value.is_linear() {
            match ArithmeticSolver::solve_fan_in_term(&value, initial_witness) {
                GateStatus::GateUnsolvable => return Err(missing_assignment(value_witness)),
                GateStatus::GateSolvable(sum, (coef, w)) => {
                    let map_value =
                        self.get_value(index).ok_or_else(|| missing_assignment(Some(w)))?;
                    insert_value(&w, (map_value - sum - value.q_c) / coef, initial_witness)?;
                }
                GateStatus::GateSatisfied(sum) => {
                    self.insert_value(index, sum + value.q_c);
                }
            }
        } else {
            return Err(missing_assignment(value_witness));
        }
        Ok(())
    }
}
