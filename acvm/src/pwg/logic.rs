use super::{insert_value, witness_to_value};
use crate::{pwg::OpcodeResolution, OpcodeResolutionError};
use acir::{circuit::opcodes::FunctionInput, native_types::Witness, FieldElement};
use std::collections::BTreeMap;

/// Solves a [`BlackBoxFunc::And`][acir::circuit::black_box_functions::BlackBoxFunc::AND] opcode and inserts
/// the result into the supplied witness map
pub fn and(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    lhs: &FunctionInput,
    rhs: &FunctionInput,
    output: &Witness,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    assert_eq!(
        lhs.num_bits, rhs.num_bits,
        "number of bits specified for each input must be the same"
    );
    solve_logic_gate(initial_witness, &lhs.witness, &rhs.witness, *output, |left, right| {
        left.and(right, lhs.num_bits)
    })
}

/// Solves a [`BlackBoxFunc::XOR`][acir::circuit::black_box_functions::BlackBoxFunc::XOR] opcode and inserts
/// the result into the supplied witness map
pub fn xor(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    lhs: &FunctionInput,
    rhs: &FunctionInput,
    output: &Witness,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    assert_eq!(
        lhs.num_bits, rhs.num_bits,
        "number of bits specified for each input must be the same"
    );
    solve_logic_gate(initial_witness, &lhs.witness, &rhs.witness, *output, |left, right| {
        left.xor(right, lhs.num_bits)
    })
}

/// Derives the rest of the witness based on the initial low level variables
fn solve_logic_gate(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    a: &Witness,
    b: &Witness,
    result: Witness,
    logic_op: impl Fn(&FieldElement, &FieldElement) -> FieldElement,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let w_l_value = witness_to_value(initial_witness, *a)?;
    let w_r_value = witness_to_value(initial_witness, *b)?;
    let assignment = logic_op(w_l_value, w_r_value);

    insert_value(&result, assignment, initial_witness)?;
    Ok(OpcodeResolution::Solved)
}
