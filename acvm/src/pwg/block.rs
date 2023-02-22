use std::collections::{BTreeMap, HashMap};

use acir::{
    circuit::opcodes::{BlockId, BlockOp},
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
        trace: &Vec<BlockOp>,
        solved_witness: &mut BTreeMap<Witness, FieldElement>,
    ) -> Result<GateResolution, OpcodeResolutionError> {
        let solver = self.blocks.entry(id).or_default();
        solver.solve(solved_witness, trace)
    }
}

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
        trace: &Vec<BlockOp>,
    ) -> Result<GateResolution, OpcodeResolutionError> {
        let mut unknown = None;
        while self.solved_operations < trace.len() {
            let op_idx = self.solved_operations;
            let op_expr = ArithmeticSolver::evaluate(&trace[op_idx].operation, initial_witness);
            if let Some(operation) = expression_to_const(&op_expr) {
                let index_expr = ArithmeticSolver::evaluate(&trace[op_idx].index, initial_witness);
                if let Some(index) = expression_to_const(&index_expr) {
                    let index = index.try_to_u64().unwrap() as u32;
                    if operation == FieldElement::zero() {
                        let value =
                            ArithmeticSolver::evaluate(&trace[op_idx].value, initial_witness);
                        unknown = value.get_witness();
                        if value.is_const() {
                            self.insert_value(index, value.q_c)?;
                        } else if value.is_linear() {
                            match ArithmeticSolver::solve_fan_in_term(&value, initial_witness) {
                                GateStatus::GateUnsolvable => break,
                                GateStatus::GateSolvable(sum, (coef, w)) => {
                                    let map_value = self.get_value(index);
                                    if let Some(map_value) = map_value {
                                        insert_witness(
                                            w,
                                            (map_value - sum - value.q_c) / coef,
                                            initial_witness,
                                        )?;
                                    } else {
                                        unknown = Some(w);
                                        break;
                                    }
                                }
                                GateStatus::GateSatisfied(sum) => {
                                    self.insert_value(index, sum + value.q_c)?;
                                }
                            }
                        } else {
                            break;
                        }
                    } else {
                        let value =
                            ArithmeticSolver::evaluate(&trace[op_idx].value, initial_witness);
                        if value.is_const() {
                            self.insert_value(index, value.q_c)?;
                        } else {
                            break;
                        }
                    }
                } else {
                    unknown = index_expr.get_witness();
                    break;
                }
            } else {
                unknown = op_expr.get_witness();
                break;
            }
            self.solved_operations += 1;
        }
        if self.solved_operations == trace.len() {
            Ok(GateResolution::Resolved)
        } else {
            Ok(GateResolution::Skip(OpcodeNotSolvable::MissingAssignment(unknown.unwrap().0)))
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use acir::{
        circuit::opcodes::{BlockId, BlockOp},
        native_types::{Expression, Witness},
        FieldElement,
    };

    use crate::pwg::directives::insert_witness;

    use super::Blocks;

    #[test]
    fn test_solver() {
        let mut index = FieldElement::zero();
        let mut trace = vec![BlockOp {
            operation: Expression::one(),
            index: Expression::from_field(index),
            value: Expression::from(&Witness(1)),
        }];
        index += FieldElement::one();
        trace.push(BlockOp {
            operation: Expression::one(),
            index: Expression::from_field(index),
            value: Expression::from(&Witness(2)),
        });
        index += FieldElement::one();
        trace.push(BlockOp {
            operation: Expression::one(),
            index: Expression::from_field(index),
            value: Expression::from(&Witness(3)),
        });
        trace.push(BlockOp {
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
