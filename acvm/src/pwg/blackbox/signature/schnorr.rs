use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    FieldElement,
};

use crate::{
    pwg::{insert_value, witness_to_value, OpcodeResolution, OpcodeResolutionError},
    BlackBoxFunctionSolver,
};

use super::to_u8_vec;

pub(crate) fn schnorr_verify(
    backend: &impl BlackBoxFunctionSolver,
    initial_witness: &mut WitnessMap,
    public_key_x: FunctionInput,
    public_key_y: FunctionInput,
    signature_s: FunctionInput,
    signature_e: FunctionInput,
    message: &[FunctionInput],
    output: Witness,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let public_key_x: &FieldElement = witness_to_value(initial_witness, public_key_x.witness)?;
    let public_key_y: &FieldElement = witness_to_value(initial_witness, public_key_y.witness)?;

    let sig_s: &FieldElement = witness_to_value(initial_witness, signature_s.witness)?;
    let sig_e: &FieldElement = witness_to_value(initial_witness, signature_e.witness)?;

    let message = to_u8_vec(initial_witness, message)?;

    let valid_signature =
        backend.schnorr_verify(public_key_x, public_key_y, sig_s, sig_e, &message)?;

    insert_value(&output, FieldElement::from(valid_signature), initial_witness)?;

    Ok(OpcodeResolution::Solved)
}
