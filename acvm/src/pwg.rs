// Re-usable methods that backends can use to implement their PWG

use crate::{OpcodeNotSolvable, OpcodeResolutionError};
use acir::{
    native_types::{Expression, Witness},
    FieldElement,
};
use std::collections::BTreeMap;

// arithmetic
pub mod arithmetic;
// Directives
pub mod directives;
// black box functions
pub mod hash;
pub mod logic;
pub mod range;
pub mod signature;
pub mod sorting;

// Returns the concrete value for a particular witness
// If the witness has no assignment, then
// an error is returned
pub fn witness_to_value(
    initial_witness: &BTreeMap<Witness, FieldElement>,
    witness: Witness,
) -> Result<&FieldElement, OpcodeResolutionError> {
    match initial_witness.get(&witness) {
        Some(value) => Ok(value),
        None => Err(OpcodeResolutionError::OpcodeNotSolvable(
            OpcodeNotSolvable::MissingAssignment(witness.0),
        )),
    }
}
// TODO: There is an issue open to decide on whether we need to get values from Expressions
// TODO versus just getting values from Witness
pub fn get_value(
    expr: &Expression,
    initial_witness: &BTreeMap<Witness, FieldElement>,
) -> Result<FieldElement, OpcodeResolutionError> {
    let mut result = expr.q_c;

    for term in &expr.linear_combinations {
        let coefficient = term.0;
        let variable = term.1;

        // Get the value assigned to that variable
        let assignment = *witness_to_value(initial_witness, variable)?;

        result += coefficient * assignment;
    }

    for term in &expr.mul_terms {
        let coefficient = term.0;
        let lhs_variable = term.1;
        let rhs_variable = term.2;

        // Get the values assigned to those variables
        let lhs_assignment = *witness_to_value(initial_witness, lhs_variable)?;
        let rhs_assignment = *witness_to_value(initial_witness, rhs_variable)?;

        result += coefficient * lhs_assignment * rhs_assignment;
    }

    Ok(result)
}
