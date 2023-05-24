use acir::{
    circuit::opcodes::FunctionInput,
    native_types::{Witness, WitnessMap},
    FieldElement,
};
use blake2::{Blake2s256, Digest};
use sha2::Sha256;
use sha3::Keccak256;

use crate::{pwg::OpcodeResolution, OpcodeResolutionError};

use super::{insert_value, witness_to_value};

pub fn blake2s256(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Blake2s256>(initial_witness, None, inputs)?;

    for (output_witness, value) in outputs.iter().zip(hash.iter()) {
        insert_value(
            output_witness,
            FieldElement::from_be_bytes_reduce(&[*value]),
            initial_witness,
        )?;
    }

    Ok(OpcodeResolution::Solved)
}

pub fn sha256(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Sha256>(initial_witness, None, inputs)?;

    for (output_witness, value) in outputs.iter().zip(hash.iter()) {
        insert_value(
            output_witness,
            FieldElement::from_be_bytes_reduce(&[*value]),
            initial_witness,
        )?;
    }

    Ok(OpcodeResolution::Solved)
}

pub fn keccak256(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Keccak256>(initial_witness, None, inputs)?;

    for (output_witness, value) in outputs.iter().zip(hash.iter()) {
        insert_value(
            output_witness,
            FieldElement::from_be_bytes_reduce(&[*value]),
            initial_witness,
        )?;
    }

    Ok(OpcodeResolution::Solved)
}

pub fn keccak256_variable_length(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    var_message_size: FunctionInput,
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Keccak256>(initial_witness, Some(var_message_size), inputs)?;

    for (output_witness, value) in outputs.iter().zip(hash.iter()) {
        insert_value(
            output_witness,
            FieldElement::from_be_bytes_reduce(&[*value]),
            initial_witness,
        )?;
    }

    Ok(OpcodeResolution::Solved)
}

pub fn hash_to_field_128_security(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    output: &Witness,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Blake2s256>(initial_witness, None, inputs)?;

    let reduced_res = FieldElement::from_be_bytes_reduce(&hash);
    insert_value(output, reduced_res, initial_witness)?;

    Ok(OpcodeResolution::Solved)
}

fn generic_hash_256<D: Digest>(
    initial_witness: &mut WitnessMap,
    num_bytes_to_truncate_message: Option<FunctionInput>,
    inputs: &[FunctionInput],
) -> Result<[u8; 32], OpcodeResolutionError> {
    let mut hasher = D::new();

    let mut message_input = Vec::new();

    // Read witness assignments into hasher.
    for input in inputs.iter() {
        let witness = input.witness;
        let num_bits = input.num_bits as usize;

        let witness_assignment = witness_to_value(initial_witness, witness)?;
        let bytes = witness_assignment.fetch_nearest_bytes(num_bits);
        message_input.extend(bytes);
    }

    // Truncate the message if there is a message_size parameter given
    match num_bytes_to_truncate_message {
        Some(input) => {
            let num_bytes_to_take =
                witness_to_value(initial_witness, input.witness)?.to_u128() as usize;

            let truncated_message = &message_input[0..num_bytes_to_take];
            hasher.update(truncated_message)
        }
        None => hasher.update(message_input),
    }

    let result = hasher.finalize().as_slice().try_into().unwrap();
    Ok(result)
}
