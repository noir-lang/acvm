use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};

use crate::{
    pwg::{insert_value, witness_to_value, OpcodeResolution, OpcodeResolutionError},
    BlackBoxFunctionSolver,
};

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

    let mut message_bytes: Vec<u8> = Vec::new();
    for msg in message.iter() {
        let msg_i_field = witness_to_value(initial_witness, msg.witness)?;
        let msg_i = *msg_i_field.to_be_bytes().last().ok_or_else(|| {
            OpcodeResolutionError::BlackBoxFunctionFailed(
                BlackBoxFunc::SchnorrVerify,
                "could not get last bytes".into(),
            )
        })?;
        message_bytes.push(msg_i);
    }

    let valid_signature =
        backend.schnorr_verify(public_key_x, public_key_y, (sig_s, sig_e), &message_bytes)?;

    insert_value(&output, FieldElement::from(valid_signature), initial_witness)?;

    Ok(OpcodeResolution::Solved)
}
