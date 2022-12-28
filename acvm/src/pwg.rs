// Re-usable methods that backends can use to implement their PWG

use crate::{OpcodeNotSolvable, OpcodeResolutionError};
use acir::{native_types::Witness, FieldElement};
use std::collections::BTreeMap;

pub mod arithmetic;
pub mod hash;
pub mod logic;
pub mod signature;

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
