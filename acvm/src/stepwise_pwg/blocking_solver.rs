use super::attempt_opcode::{attempt_opcode, AttemptOpcodeOutcome};
use crate::OpcodeResolutionError;
use acir::{
    circuit::{opcodes::BlackBoxFuncCall, Opcode},
    native_types::Witness,
    FieldElement,
};
use std::collections::BTreeMap;

pub struct BlockedByBlackBoxFuncCall {
    pub black_box_func_call: BlackBoxFuncCall,
    pub unsolved_opcodes: Vec<Opcode>,
}

pub enum StepOutcome {
    BlockedByBlackBoxFuncCall(BlockedByBlackBoxFuncCall),
    Done,
}

pub struct BlockingSolver;

impl BlockingSolver {
    pub fn solve_until_blocked(
        witness_skeleton: &mut BTreeMap<Witness, FieldElement>,
        opcodes: &mut Vec<Opcode>,
    ) -> Result<StepOutcome, OpcodeResolutionError> {
        if opcodes.is_empty() {
            return Ok(StepOutcome::Done);
        }
        let mut unsolved_opcodes: Vec<Opcode> = Vec::new();

        let mut blocking_blackbox_call: Option<BlackBoxFuncCall> = None;
        for opcode in opcodes.drain(..) {
            if blocking_blackbox_call.is_some() {
                // Skipping remaining opcodes since we are now blocked
                unsolved_opcodes.push(opcode);
            } else {
                match attempt_opcode(witness_skeleton, opcode) {
                    AttemptOpcodeOutcome::Solved => {
                        // We do nothing in the happy case
                    }
                    AttemptOpcodeOutcome::Skipped(opcode) => {
                        // For opcode not solvable errors, we push those opcodes to the back as
                        // it could be because the opcodes are out of order, ie this assignment
                        // relies on a later opcodes's results
                        unsolved_opcodes.push(opcode);
                    }
                    AttemptOpcodeOutcome::Blocked(bb_func_call) => {
                        blocking_blackbox_call = Some(bb_func_call);
                    }
                    AttemptOpcodeOutcome::Err(err) => return Err(err),
                }
            }
        }
        if let Some(black_box_func_call) = blocking_blackbox_call {
            Ok(StepOutcome::BlockedByBlackBoxFuncCall(
                BlockedByBlackBoxFuncCall {
                    black_box_func_call,
                    unsolved_opcodes,
                },
            ))
        } else {
            // Recurse to reattempt skipped opcodes
            Self::solve_until_blocked(witness_skeleton, &mut unsolved_opcodes)
        }
    }
}

#[cfg(test)]
mod tests {
    use acir::{
        circuit::{
            opcodes::{BlackBoxFuncCall, FunctionInput},
            Opcode,
        },
        native_types::{Expression, Witness},
        BlackBoxFunc, FieldElement,
    };
    use std::collections::BTreeMap;

    use super::{BlockingSolver, StepOutcome};

    #[test]
    fn solve_until_blocked_smoke_test() {
        let mut opcodes0 = vec![
            // Deliberately ordered incorrectly
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                name: BlackBoxFunc::Pedersen,
                inputs: vec![FunctionInput {
                    witness: Witness(1),
                    num_bits: 32,
                }],
                outputs: vec![Witness(2)],
            }),
            Opcode::Arithmetic(Expression {
                mul_terms: vec![],
                linear_combinations: vec![
                    (FieldElement::one(), Witness(0)),
                    (FieldElement::one(), Witness(1)),
                ],
                q_c: FieldElement::zero(),
            }),
        ];
        let mut witness_skeleton = BTreeMap::from([(Witness(0), FieldElement::one())]);
        let outcome = BlockingSolver::solve_until_blocked(&mut witness_skeleton, &mut opcodes0);
        match outcome {
            Ok(StepOutcome::BlockedByBlackBoxFuncCall(blocked_by)) => {
                assert!(
                    blocked_by.unsolved_opcodes.is_empty(),
                    "The above expression is solvable, leaving just the black box."
                )
            }
            _ => panic!("Should be blocked"),
        };
    }
}
