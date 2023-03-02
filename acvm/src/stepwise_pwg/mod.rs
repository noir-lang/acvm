mod attempt_blackbox;
mod attempt_opcode;
mod blocking_solver;
use crate::{
    stepwise_pwg::blocking_solver::{BlockingSolver, StepOutcome},
    OpcodeResolutionError,
};
use acir::{
    circuit::{opcodes::BlackBoxFuncCall, Opcode},
    native_types::Witness,
    BlackBoxFunc, FieldElement,
};
use std::collections::BTreeMap;
use thiserror::Error;

pub struct StepwisePartialWitnessGenerator {
    partial_witness: BTreeMap<Witness, FieldElement>,
    unsolved_opcodes: Vec<Opcode>,
    blocking_blackbox_func_call: Option<BlackBoxFuncCall>,
}

pub struct BlackBoxCallResolvedInputs {
    pub name: BlackBoxFunc,
    pub inputs: Vec<FieldElement>,
}

#[derive(PartialEq, Eq, Debug, Error)]
pub enum StepwisePwgError {
    #[error("cannot step until blocking black box is solved")]
    Blocked,
    #[error("no blocking black box function call needs solving")]
    NotBlocked,
    #[error("expected black box solution length of {0}, but received {1}")]
    BadSolutionLength(usize, usize),
    #[error("step failed with OpcodeResolutionError: {0}")]
    OpcodeResolutionError(OpcodeResolutionError),
    #[error("solving isn't finished")]
    NotDone,
}

impl StepwisePartialWitnessGenerator {
    pub fn new(initial_witness: BTreeMap<Witness, FieldElement>, opcodes: Vec<Opcode>) -> Self {
        Self {
            partial_witness: initial_witness,
            unsolved_opcodes: opcodes,
            blocking_blackbox_func_call: None,
        }
    }

    pub fn apply_blackbox_call_solution(
        &mut self,
        solution: Vec<FieldElement>,
    ) -> Result<(), StepwisePwgError> {
        match &self.blocking_blackbox_func_call {
            None => Err(StepwisePwgError::NotBlocked),
            Some(bb_func_call) => {
                let expected_len = bb_func_call.outputs.len();
                let actual_len = solution.len();
                if expected_len != actual_len {
                    Err(StepwisePwgError::BadSolutionLength(
                        expected_len,
                        actual_len,
                    ))
                } else {
                    for (value, witness) in solution.iter().zip(bb_func_call.outputs.iter()) {
                        self.partial_witness.insert(*witness, *value);
                    }
                    self.blocking_blackbox_func_call = None;
                    Ok(())
                }
            }
        }
    }

    pub fn step(&mut self) -> Result<(), StepwisePwgError> {
        if self.blocking_blackbox_func_call.is_some() {
            return Err(StepwisePwgError::Blocked);
        }
        let result = BlockingSolver::solve_until_blocked(
            &mut self.partial_witness,
            &mut self.unsolved_opcodes,
        );

        match result {
            Ok(StepOutcome::BlockedByBlackBoxFuncCall(blocked_by)) => {
                self.blocking_blackbox_func_call = Some(blocked_by.black_box_func_call);
                self.unsolved_opcodes = blocked_by.unsolved_opcodes;
                Ok(())
            }
            Ok(StepOutcome::Done) => Ok(()),
            Err(err) => Err(StepwisePwgError::OpcodeResolutionError(err)),
        }
    }

    pub fn is_done(&self) -> bool {
        self.unsolved_opcodes.is_empty() && self.blocking_blackbox_func_call.is_none()
    }

    pub fn intermediate_witness(self) -> Result<BTreeMap<Witness, FieldElement>, StepwisePwgError> {
        if self.is_done() {
            Ok(self.partial_witness)
        } else {
            Err(StepwisePwgError::NotDone)
        }
    }

    pub fn required_black_box_func_call(&self) -> Option<BlackBoxCallResolvedInputs> {
        match &self.blocking_blackbox_func_call {
            Some(bb_call) => Some(BlackBoxCallResolvedInputs {
                name: bb_call.name,
                inputs: bb_call
                    .inputs
                    .iter()
                    .map(|input| self.partial_witness.get(&input.witness).unwrap().clone())
                    .collect(),
            }),
            None => None,
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

    use crate::StepwisePartialWitnessGenerator;

    #[test]
    fn stateful_partial_witness_generator_smoke_test() {
        let opcodes = vec![
            // Deliberately ordered incorrectly
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall {
                name: BlackBoxFunc::Pedersen,
                inputs: vec![FunctionInput {
                    witness: Witness(1),
                    num_bits: 32,
                }],
                outputs: vec![Witness(2), Witness(3)],
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
        let initial_witness = BTreeMap::from([(Witness(0), FieldElement::zero())]);
        let mut spwg = StepwisePartialWitnessGenerator::new(initial_witness, opcodes);
        assert!(!spwg.is_done(), "Not started");
        spwg.step().unwrap();
        assert!(!spwg.is_done(), "Hits backbox");
        let required_call = spwg.required_black_box_func_call().unwrap();
        assert_eq!(required_call.name, BlackBoxFunc::Pedersen);
        assert_eq!(
            required_call.inputs.len(),
            1,
            "This acir hashes a single field"
        );
        spwg.apply_blackbox_call_solution(vec![FieldElement::zero(), FieldElement::zero()])
            .unwrap();
        assert!(spwg.is_done(), "Nothing left to solve");
        let expected_solution = BTreeMap::from([
            (Witness(0), FieldElement::zero()),
            (Witness(1), FieldElement::zero()),
            (Witness(2), FieldElement::zero()),
            (Witness(3), FieldElement::zero()),
        ]);
        assert_eq!(
            spwg.intermediate_witness().unwrap(),
            expected_solution,
            "Solution is complete"
        )
    }
}
