use std::collections::BTreeMap;

use acir::{circuit::opcodes::OracleData, native_types::Witness, FieldElement};

use crate::{OpcodeNotSolvable, OpcodeResolution, OpcodeResolutionError};

use super::{arithmetic::ArithmeticSolver, directives::insert_witness};

pub struct OracleSolver;

impl OracleSolver {
    /// Derives the rest of the witness based on the initial low level variables
    pub fn solve(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        data: &mut OracleData,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        // Set input values
        for input in data.inputs.iter().skip(data.input_values.len()) {
            let solve = ArithmeticSolver::evaluate(input, initial_witness);
            if let Some(value) = solve.to_const() {
                data.input_values.push(value);
            } else {
                break;
            }
        }

        // If all of the inputs to the oracle have assignments
        if data.input_values.len() == data.inputs.len() {
            if data.output_values.len() == data.outputs.len() {
                for (out, value) in data.outputs.iter().zip(data.output_values.iter()) {
                    insert_witness(*out, *value, initial_witness)?;
                }
                Ok(OpcodeResolution::Solved)
            } else {
                // Missing output values
                Ok(OpcodeResolution::InProgress)
            }
        } else {
            Ok(OpcodeResolution::Stalled(OpcodeNotSolvable::ExpressionHasTooManyUnknowns(
                data.inputs
                    .last()
                    .expect("Infallible: cannot reach this point if no inputs")
                    .clone(),
            )))
        }
    }
}
