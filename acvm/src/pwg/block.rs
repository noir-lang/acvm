use std::collections::{BTreeMap, HashMap};

use acir::{
    circuit::opcodes::{BlockId, MemOp},
    native_types::Witness,
    FieldElement,
};

use crate::{GateResolution, OpcodeNotSolvable, OpcodeResolutionError};

use super::{
    arithmetic::{ArithmeticSolver, GateStatus},
    directives::insert_witness,
    expression_to_const,
};

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
    ) -> Result<GateResolution, OpcodeResolutionError> {
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
        let entry = self.block_value.entry(index);
        match entry {
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(value);
            }
            std::collections::hash_map::Entry::Occupied(e) => {
                if e.get() != &value {
                    return Err(OpcodeResolutionError::UnsatisfiedConstrain);
                }
            }
        }
        Ok(())
    }

    fn get_value(&self, index: u32) -> Option<FieldElement> {
        self.block_value.get(&index).copied()
    }

    // Try to solve block operations from the trace
    // As long as operations are resolved, we update/read from the block_value
    // We stop when an operation cannot be resolved
    pub fn solve(
        &mut self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        trace: &[MemOp],
    ) -> Result<GateResolution, OpcodeResolutionError> {
        for block_op in trace.iter().skip(self.solved_operations) {
            let op_expr = ArithmeticSolver::evaluate(&block_op.operation, initial_witness);
            if let Some(operation) = expression_to_const(&op_expr) {
                let index_expr = ArithmeticSolver::evaluate(&block_op.index, initial_witness);
                if let Some(index) = expression_to_const(&index_expr) {
                    let index = index.try_to_u64().unwrap() as u32;
                    let value = ArithmeticSolver::evaluate(&block_op.value, initial_witness);
                    let value_witness = value.get_witness();
                    if operation.is_zero() {
                        let value = ArithmeticSolver::evaluate(&block_op.value, initial_witness);
                        if value.is_const() {
                            self.insert_value(index, value.q_c)?;
                        } else if value.is_linear() {
                            match ArithmeticSolver::solve_fan_in_term(&value, initial_witness) {
                                GateStatus::GateUnsolvable => {
                                    return Ok(GateResolution::Skip(
                                        OpcodeNotSolvable::MissingAssignment(
                                            value_witness.unwrap().0,
                                        ),
                                    ))
                                }
                                GateStatus::GateSolvable(sum, (coef, w)) => {
                                    let map_value = self.get_value(index);
                                    if let Some(map_value) = map_value {
                                        insert_witness(
                                            w,
                                            (map_value - sum - value.q_c) / coef,
                                            initial_witness,
                                        )?;
                                    } else {
                                        return Ok(GateResolution::Skip(
                                            OpcodeNotSolvable::MissingAssignment(w.0),
                                        ));
                                    }
                                }
                                GateStatus::GateSatisfied(sum) => {
                                    self.insert_value(index, sum + value.q_c)?;
                                }
                            }
                        } else {
                            return Ok(GateResolution::Skip(OpcodeNotSolvable::MissingAssignment(
                                value_witness.unwrap().0,
                            )));
                        }
                    } else if value.is_const() {
                        self.insert_value(index, value.q_c)?;
                    } else {
                        return Ok(GateResolution::Skip(OpcodeNotSolvable::MissingAssignment(
                            value_witness.unwrap().0,
                        )));
                    }
                } else {
                    return Ok(GateResolution::Skip(OpcodeNotSolvable::MissingAssignment(
                        index_expr.get_witness().unwrap().0,
                    )));
                }
            } else {
                return Ok(GateResolution::Skip(OpcodeNotSolvable::MissingAssignment(
                    op_expr.get_witness().unwrap().0,
                )));
            }
            self.solved_operations += 1;
        }
        Ok(GateResolution::Resolved)
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
