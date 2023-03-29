// Re-usable methods that backends can use to implement their PWG

use crate::{OpcodeNotSolvable, OpcodeResolutionError};
use acir::{
    native_types::{Expression, Witness},
    FieldElement,
};
use std::collections::BTreeMap;

use self::arithmetic::ArithmeticSolver;

// arithmetic
pub mod arithmetic;
// Brillig bytecode
pub mod brillig;
// Directives
pub mod directives;
// black box functions
pub mod block;
pub mod hash;
pub mod logic;
pub mod oracle;
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
        None => Err(OpcodeNotSolvable::MissingAssignment(witness.0).into()),
    }
}

// TODO: There is an issue open to decide on whether we need to get values from Expressions
// TODO versus just getting values from Witness
pub fn get_value(
    expr: &Expression,
    initial_witness: &BTreeMap<Witness, FieldElement>,
) -> Result<FieldElement, OpcodeResolutionError> {
    let expr = ArithmeticSolver::evaluate(expr, initial_witness);
    match expr.to_const() {
        Some(value) => Ok(value),
        None => {
            Err(OpcodeResolutionError::OpcodeNotSolvable(OpcodeNotSolvable::MissingAssignment(
                ArithmeticSolver::any_witness_from_expression(&expr).unwrap().0,
            )))
        }
    }
}

// Inserts `value` into the initial witness map
// under the key of `witness`.
// Returns an error, if there was already a value in the map
// which does not match the value that one is about to insert
fn insert_value(
    witness: &Witness,
    value_to_insert: FieldElement,
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
) -> Result<(), OpcodeResolutionError> {
    let optional_old_value = initial_witness.insert(*witness, value_to_insert);

    let old_value = match optional_old_value {
        Some(old_value) => old_value,
        None => return Ok(()),
    };

    if old_value != value_to_insert {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain);
    }

    Ok(())
}
