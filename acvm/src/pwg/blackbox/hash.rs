use acir::{
    circuit::{opcodes::FunctionInput, OpcodeLabel},
    native_types::{Witness, WitnessMap},
    BlackBoxFunc, FieldElement,
};
use blake2::{Blake2s256, Digest};
use sha2::Sha256;
use sha3::Keccak256;

use crate::pwg::{insert_value, witness_to_value};
use crate::{pwg::OpcodeResolution, OpcodeResolutionError};

/// Returns the sha256 hash of the provided `data`.
pub(crate) fn sha256(data: &[u8]) -> [u8; 32] {
    generic_hash_256::<Sha256>(data)
}

/// Returns the blake2s hash of the provided `data`.
pub(crate) fn blake2s256(data: &[u8]) -> [u8; 32] {
    generic_hash_256::<Blake2s256>(data)
}

/// Returns the keccak256 hash of the provided `data`.
pub(crate) fn keccak256(data: &[u8]) -> [u8; 32] {
    generic_hash_256::<Keccak256>(data)
}

/// Hashes `data` into a 32 byte digest.
fn generic_hash_256<D: Digest>(data: &[u8]) -> [u8; 32] {
    D::digest(data).as_slice().try_into().expect("digest should be 256 bits")
}

/// Attempts to solve a `HashToField128Security` opcode
/// If successful, `initial_witness` will be mutated to contain the new witness assignment.
pub(super) fn hash_to_field_128_security(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    output: &Witness,
    opcode_idx: OpcodeLabel,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let message_input = get_hash_input(initial_witness, inputs, None)?;
    let digest = blake2s256(&message_input);

    let reduced_res = FieldElement::from_be_bytes_reduce(&digest);
    insert_value(output, reduced_res, initial_witness, opcode_idx)?;

    Ok(OpcodeResolution::Solved)
}

/// Attempts to solve a 256 bit hash function opcode.
/// If successful, `initial_witness` will be mutated to contain the new witness assignment.
pub(super) fn solve_generic_256_hash_opcode(
    initial_witness: &mut WitnessMap,
    inputs: &[FunctionInput],
    var_message_size: Option<&FunctionInput>,
    outputs: &[Witness],
    hash_function: fn(data: &[u8]) -> [u8; 32],
    black_box_func: BlackBoxFunc,
    opcode_idx: OpcodeLabel,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let message_input = get_hash_input(initial_witness, inputs, var_message_size)?;
    let digest: [u8; 32] = hash_function(&message_input);

    let outputs: [Witness; 32] = outputs.try_into().map_err(|_| {
        OpcodeResolutionError::BlackBoxFunctionFailed(
            black_box_func,
            format!("Expected 32 outputs but encountered {}", outputs.len()),
        )
    })?;
    write_digest_to_outputs(initial_witness, outputs, digest, opcode_idx)?;

    Ok(OpcodeResolution::Solved)
}

/// Reads the hash function input from a [`WitnessMap`].
fn get_hash_input(
    initial_witness: &WitnessMap,
    inputs: &[FunctionInput],
    message_size: Option<&FunctionInput>,
) -> Result<Vec<u8>, OpcodeResolutionError> {
    // Read witness assignments.
    let mut message_input = Vec::new();
    for input in inputs.iter() {
        let witness = input.witness;
        let num_bits = input.num_bits as usize;

        let witness_assignment = witness_to_value(initial_witness, witness)?;
        let bytes = witness_assignment.fetch_nearest_bytes(num_bits);
        message_input.extend(bytes);
    }

    // Truncate the message if there is a `message_size` parameter given
    match message_size {
        Some(input) => {
            let num_bytes_to_take =
                witness_to_value(initial_witness, input.witness)?.to_u128() as usize;

            // If the number of bytes to take is more than the amount of bytes available
            // in the message, then we error.
            if num_bytes_to_take > message_input.len() {
                return Err(OpcodeResolutionError::BlackBoxFunctionFailed(
                        acir::BlackBoxFunc::Keccak256,
                        format!("the number of bytes to take from the message is more than the number of bytes in the message. {} > {}", num_bytes_to_take, message_input.len()),
                    ));
            }
            let truncated_message = message_input[0..num_bytes_to_take].to_vec();
            Ok(truncated_message)
        }
        None => Ok(message_input),
    }
}

/// Writes a `digest` to the [`WitnessMap`] at witness indices `outputs`.
fn write_digest_to_outputs(
    initial_witness: &mut WitnessMap,
    outputs: [Witness; 32],
    digest: [u8; 32],
    opcode_idx: OpcodeLabel,
) -> Result<(), OpcodeResolutionError> {
    for (output_witness, value) in outputs.iter().zip(digest.into_iter()) {
        insert_value(
            output_witness,
            FieldElement::from_be_bytes_reduce(&[value]),
            initial_witness,
            opcode_idx,
        )?;
    }

    Ok(())
}
