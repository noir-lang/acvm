use acir::{circuit::opcodes::OracleData, native_types::WitnessMap};

use super::{arithmetic::ArithmeticSolver, insert_value};
use super::{OpcodeNotSolvable, OpcodeResolution, OpcodeResolutionError};

pub(super) struct OracleSolver;

impl OracleSolver {
    /// Derives the rest of the witness based on the initial low level variables
    pub(super) fn solve(
        initial_witness: &mut WitnessMap,
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
                    insert_value(out, *value, initial_witness)?;
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
