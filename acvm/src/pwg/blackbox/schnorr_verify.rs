use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};

use crate::{
    pwg::{insert_value, witness_to_value, OpcodeResolution, OpcodeResolutionError},
    BlackBoxFunctionSolver,
};

fn to_u8_vec(
    initial_witness: &WitnessMap,
    inputs: &[FunctionInput],
) -> Result<Vec<u8>, OpcodeResolutionError> {
    let mut result = Vec::with_capacity(inputs.len());
    for input in inputs {
        let witness_value_bytes = witness_to_value(initial_witness, input.witness)?.to_be_bytes();
        let byte = witness_value_bytes
            .last()
            .expect("Field element must be represented by non-zero amount of bytes");
        result.push(*byte);
    }
    Ok(result)
}

pub(super) fn schnorr_verify(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    public_key_x: FunctionInput,
    public_key_y: FunctionInput,
    signature: (FunctionInput, FunctionInput),
    message: &[FunctionInput],
    output: Witness,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let public_key_x: &FieldElement = witness_to_value(initial_witness, public_key_x.witness)?;
    let public_key_y: &FieldElement = witness_to_value(initial_witness, public_key_y.witness)?;

    let sig_s: &FieldElement = witness_to_value(initial_witness, signature.0.witness)?;
    let sig_e: &FieldElement = witness_to_value(initial_witness, signature.1.witness)?;

    let message = to_u8_vec(initial_witness, message)?;

    let valid_signature =
        backend.schnorr_verify(public_key_x, public_key_y, (sig_s, sig_e), &message_bytes)?;

    insert_value(&output, FieldElement::from(valid_signature), initial_witness)?;

    Ok(OpcodeResolution::Solved)
}
