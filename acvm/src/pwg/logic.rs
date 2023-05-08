use super::{insert_value, witness_to_value};
use crate::{pwg::OpcodeResolution, OpcodeResolutionError};
use acir::{circuit::opcodes::FunctionInput, native_types::Witness, FieldElement};
use std::collections::BTreeMap;

/// Solves a [`BlackBoxFunc::And`][acir::circuit::black_box_functions::BlackBoxFunc::AND] opcode and inserts
/// the result into the supplied witness map
pub fn and(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let (a, b, result, num_bits) = extract_input_output(inputs, outputs);
    solve_logic_gate(initial_witness, &a, &b, result, |left, right| left.and(right, num_bits))
}

/// Solves a [`BlackBoxFunc::XOR`][acir::circuit::black_box_functions::BlackBoxFunc::XOR] opcode and inserts
/// the result into the supplied witness map
pub fn xor(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let (a, b, result, num_bits) = extract_input_output(inputs, outputs);
    solve_logic_gate(initial_witness, &a, &b, result, |left, right| left.xor(right, num_bits))
}

// TODO: Is there somewhere else that we can put this?
// TODO: extraction methods are needed for some opcodes like logic and range
pub(crate) fn extract_input_output(
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> (Witness, Witness, Witness, u32) {
    let a = inputs[0];
    let b = inputs[1];
    let result = outputs[0];

    // The num_bits variable should be the same for all witnesses
    assert_eq!(a.num_bits, b.num_bits, "number of bits specified for each input must be the same");

    let num_bits = a.num_bits;

    (a.witness, b.witness, result, num_bits)
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
