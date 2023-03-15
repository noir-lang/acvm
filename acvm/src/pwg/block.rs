use std::collections::{BTreeMap, HashMap};

use acir::{
    circuit::opcodes::{BlockId, MemOp},
    native_types::Witness,
    FieldElement,
};

use crate::{OpcodeNotSolvable, OpcodeResolution, OpcodeResolutionError};

use super::{
    arithmetic::{ArithmeticSolver, GateStatus},
    directives::insert_witness,
};

/// Maps a block to its emulated state
#[derive(Default)]
pub struct Blocks {
    blocks: HashMap<BlockId, BlockSolver>,
}

impl Blocks {
    pub fn solve(
        &mut self,
        id: BlockId,
        trace: &[MemOp],
        solved_witness: &mut BTreeMap<Witness, FieldElement>,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        let solver = self.blocks.entry(id).or_default();
        solver.solve(solved_witness, trace)
    }
}

/// Maintains the state for solving Block opcode
/// block_value is the value of the Block at the solved_operations step
/// solved_operations is the number of solved elements in the block
#[derive(Default)]
struct BlockSolver {
    block_value: HashMap<u32, FieldElement>,
    solved_operations: usize,
}

impl BlockSolver {
    fn insert_value(
        &mut self,
        index: u32,
        value: FieldElement,
    ) -> Result<(), OpcodeResolutionError> {
        match self.block_value.insert(index, value) {
            Some(existing_value) if value != existing_value => {
                Err(OpcodeResolutionError::UnsatisfiedConstrain)
            }
            _ => Ok(()),
        }
    }

    fn get_value(&self, index: u32) -> Option<FieldElement> {
        self.block_value.get(&index).copied()
    }

    // Helper function which tries to solve a Block opcode
    // As long as operations are resolved, we update/read from the block_value
    // We stop when an operation cannot be resolved
    fn solve_helper(
        &mut self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        trace: &[MemOp],
    ) -> Result<(), OpcodeResolutionError> {
        let missing_assignment = |witness: Option<Witness>| {
            OpcodeResolutionError::OpcodeNotSolvable(OpcodeNotSolvable::MissingAssignment(
                witness.unwrap().0,
            ))
        };

        for block_op in trace.iter().skip(self.solved_operations) {
            let op_expr = ArithmeticSolver::evaluate(&block_op.operation, initial_witness);
            let operation = op_expr.to_const().ok_or_else(|| {
                missing_assignment(ArithmeticSolver::any_witness_from_expression(&op_expr))
            })?;
            let index_expr = ArithmeticSolver::evaluate(&block_op.index, initial_witness);
            let index = index_expr.to_const().ok_or_else(|| {
                missing_assignment(ArithmeticSolver::any_witness_from_expression(&index_expr))
            })?;
            let index = index.try_to_u64().unwrap() as u32;
            let value = ArithmeticSolver::evaluate(&block_op.value, initial_witness);
            let value_witness = ArithmeticSolver::any_witness_from_expression(&value);
            if value.is_const() {
                self.insert_value(index, value.q_c)?;
            } else if operation.is_zero() && value.is_linear() {
                match ArithmeticSolver::solve_fan_in_term(&value, initial_witness) {
                    GateStatus::GateUnsolvable => return Err(missing_assignment(value_witness)),
                    GateStatus::GateSolvable(sum, (coef, w)) => {
                        let map_value = self.get_value(index).ok_or(missing_assignment(Some(w)))?;
                        insert_witness(w, (map_value - sum - value.q_c) / coef, initial_witness)?;
                    }
                    GateStatus::GateSatisfied(sum) => {
                        self.insert_value(index, sum + value.q_c)?;
                    }
                }
            } else {
                return Err(missing_assignment(value_witness));
            }
            self.solved_operations += 1;
        }
        Ok(())
    }

    // Try to solve block operations from the trace
    // The function calls solve_helper() for solving the opcode
    // and converts its result into GateResolution
    pub(crate) fn solve(
        &mut self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
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
mod test {
    use std::collections::BTreeMap;

    use acir::{
        circuit::opcodes::{BlockId, MemOp},
        native_types::{Expression, Witness},
        FieldElement,
    };

    use crate::pwg::directives::insert_witness;

    use super::Blocks;

    #[test]
    fn test_solver() {
        let mut index = FieldElement::zero();
        let mut trace = vec![MemOp {
            operation: Expression::one(),
            index: Expression::from_field(index),
            value: Expression::from(&Witness(1)),
        }];
        index += FieldElement::one();
        trace.push(MemOp {
            operation: Expression::one(),
            index: Expression::from_field(index),
            value: Expression::from(&Witness(2)),
        });
        index += FieldElement::one();
        trace.push(MemOp {
            operation: Expression::one(),
            index: Expression::from_field(index),
            value: Expression::from(&Witness(3)),
        });
        trace.push(MemOp {
            operation: Expression::zero(),
            index: Expression::one(),
            value: Expression::from(&Witness(4)),
        });
        let id = BlockId::default();
        let mut initial_witness = BTreeMap::new();
        let mut value = FieldElement::zero();
        insert_witness(Witness(1), value, &mut initial_witness).unwrap();
        value = FieldElement::one();
        insert_witness(Witness(2), value, &mut initial_witness).unwrap();
        value = value + value;
        insert_witness(Witness(3), value, &mut initial_witness).unwrap();
        let mut blocks = Blocks::default();
        blocks.solve(id, &mut trace, &mut initial_witness).unwrap();
        assert_eq!(initial_witness[&Witness(4)], FieldElement::one());
    }
}
