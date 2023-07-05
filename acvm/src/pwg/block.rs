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
use super::{OpcodeNotSolvable, OpcodeResolution, OpcodeResolutionError};

/// Maintains the state for solving Block opcode
/// block_value is the value of the Block at the solved_operations step
/// solved_operations is the number of solved elements in the block
#[derive(Default)]
pub(super) struct BlockSolver;

impl BlockSolver {
    // Helper function which tries to solve a Block opcode
    // As long as operations are resolved, we update/read from the block_value
    // We stop when an operation cannot be resolved
    fn solve_helper(
        &mut self,
        initial_witness: &mut WitnessMap,
        trace: &[MemOp],
    ) -> Result<(), OpcodeResolutionError> {
        let missing_assignment = |witness: Option<Witness>| {
            OpcodeResolutionError::OpcodeNotSolvable(OpcodeNotSolvable::MissingAssignment(
                witness.unwrap().0,
            ))
        };

        let mut block_value: HashMap<u32, FieldElement> = HashMap::new();

        for block_op in trace.iter() {
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
                block_value.insert(index, value.q_c);
            } else if operation.is_zero() && value.is_linear() {
                match ArithmeticSolver::solve_fan_in_term(&value, initial_witness) {
                    GateStatus::GateUnsolvable => return Err(missing_assignment(value_witness)),
                    GateStatus::GateSolvable(sum, (coef, w)) => {
                        let map_value = block_value
                            .get(&index)
                            .copied()
                            .ok_or_else(|| missing_assignment(Some(w)))?;
                        insert_value(&w, (map_value - sum - value.q_c) / coef, initial_witness)?;
                    }
                    GateStatus::GateSatisfied(sum) => {
                        block_value.insert(index, sum + value.q_c);
                    }
                }
            } else {
                return Err(missing_assignment(value_witness));
            }
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
        match self.solve_helper(initial_witness, trace) {
            Ok(()) => Ok(OpcodeResolution::Solved),
            Err(OpcodeResolutionError::OpcodeNotSolvable(err)) => {
                Ok(OpcodeResolution::Stalled(err))
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
