use acir::{circuit::opcodes::FunctionInput, native_types::Witness, FieldElement};
use blake2::{Blake2s256, Digest};
use sha2::Sha256;
use sha3::Keccak256;
use std::collections::BTreeMap;

use crate::{pwg::OpcodeResolution, OpcodeResolutionError};

use super::{insert_value, witness_to_value};

pub fn blake2s256(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Blake2s256>(initial_witness, inputs)?;

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
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Sha256>(initial_witness, inputs)?;

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
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Keccak256>(initial_witness, inputs)?;

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
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
    outputs: &[Witness],
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Blake2s256>(initial_witness, inputs)?;

    let reduced_res = FieldElement::from_be_bytes_reduce(&hash);
    insert_value(&outputs[0], reduced_res, initial_witness)?;

    Ok(OpcodeResolution::Solved)
}

fn generic_hash_256<D: Digest>(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
) -> Result<[u8; 32], OpcodeResolutionError> {
    let mut hasher = D::new();

    // Read witness assignments into hasher.
    for input in inputs.iter() {
        let witness = input.witness;
        let num_bits = input.num_bits as usize;

        let witness_assignment = witness_to_value(initial_witness, witness)?;
        let bytes = witness_assignment.fetch_nearest_bytes(num_bits);
        hasher.update(bytes);
    }

    let result = hasher.finalize().as_slice().try_into().unwrap();
    Ok(result)
}
