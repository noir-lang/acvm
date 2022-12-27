use acir::{circuit::opcodes::FunctionInput, native_types::Witness, FieldElement};
use std::collections::BTreeMap;

// Re-usable methods that backends can use to implement their PWG
// XXX: This can possible be refactored to be default trait methods

pub mod arithmetic;
pub mod hash;
pub mod logic;
pub mod signature;

pub fn input_to_value<'a>(
    witness_map: &'a BTreeMap<Witness, FieldElement>,
    input: &FunctionInput,
) -> &'a FieldElement {
    match witness_map.get(&input.witness) {
        None => panic!("Cannot find witness assignment for {:?}", input),
        Some(assignment) => assignment,
    }
}
