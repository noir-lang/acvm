use acir::{circuit::opcodes::OracleData, native_types::WitnessMap, FieldElement};

use crate::{pwg::OpcodeResolution, OpcodeNotSolvable, OpcodeResolutionError};

use super::{arithmetic::ArithmeticSolver, get_value, insert_value};

pub struct OracleSolver;

impl OracleSolver {
    /// Derives the rest of the witness based on the initial low level variables
    pub fn solve(
        initial_witness: &mut WitnessMap,
        data: &mut OracleData,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        // If the predicate is `None`, then we simply return the value 1
        // If the predicate is `Some` but we cannot find a value, then we return stalled
        let pred_value = match &data.predicate {
            Some(pred) => get_value(pred, initial_witness),
            None => Ok(FieldElement::one()),
        };
        let pred_value = match pred_value {
            Ok(pred_value) => pred_value,
            Err(OpcodeResolutionError::OpcodeNotSolvable(unsolved)) => {
                return Ok(OpcodeResolution::Stalled(unsolved))
            }
            Err(err) => return Err(err),
        };

        // A zero predicate indicates the oracle should be skipped, and its outputs zeroed.
        if pred_value.is_zero() {
            for output in &data.outputs {
                insert_value(output, FieldElement::zero(), initial_witness)?;
            }
            return Ok(OpcodeResolution::Solved);
        }

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
