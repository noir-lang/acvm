use acir::{circuit::opcodes::BlackBoxFuncCall, native_types::Witness, FieldElement};
use blake2::{Blake2s, Digest};
use sha2::Sha256;
use sha3::Keccak256;
use std::collections::BTreeMap;

use crate::{OpcodeResolution, OpcodeResolutionError};

use super::{insert_value, witness_to_value};

pub fn blake2s(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    func_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Blake2s>(initial_witness, func_call)?;

    for (output_witness, value) in func_call.outputs.iter().zip(hash.iter()) {
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
    func_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Sha256>(initial_witness, func_call)?;

    for (output_witness, value) in func_call.outputs.iter().zip(hash.iter()) {
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
    func_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let hash = generic_hash_256::<Blake2s>(initial_witness, func_call)?;

    let reduced_res = FieldElement::from_be_bytes_reduce(&hash);
    insert_value(&func_call.outputs[0], reduced_res, initial_witness)?;

    Ok(OpcodeResolution::Solved)
}

fn generic_hash_256<D: Digest>(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    func_call: &BlackBoxFuncCall,
) -> Result<[u8; 32], OpcodeResolutionError> {
    let mut hasher = D::new();

    // Read witness assignments into hasher.
    for input in func_call.inputs.iter() {
        let witness = input.witness;
        let num_bits = input.num_bits as usize;

        let witness_assignment = witness_to_value(initial_witness, witness)?;
        let bytes = witness_assignment.fetch_nearest_bytes(num_bits);
        hasher.update(bytes);
    }

    let result = hasher.finalize().as_slice().try_into().unwrap();
    Ok(result)
}

pub fn keccak256(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    gadget_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    generic_sha3::<Keccak256>(initial_witness, gadget_call)?;
    Ok(OpcodeResolution::Solved)
}

fn generic_sha3<D: sha3::Digest>(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    gadget_call: &BlackBoxFuncCall,
) -> Result<(), OpcodeResolutionError> {
    let mut hasher = D::new();

    // For each input in the vector of inputs, check if we have their witness assignments (Can do this outside of match, since they all have inputs)
    for input_index in gadget_call.inputs.iter() {
        let witness = &input_index.witness;
        let num_bits = input_index.num_bits;

        let witness_assignment = initial_witness.get(witness);
        let assignment = match witness_assignment {
            None => panic!("cannot find witness assignment for {witness:?}"),
            Some(assignment) => assignment,
        };

        let bytes = assignment.fetch_nearest_bytes(num_bits as usize);
        hasher.update(bytes);
    }
    let result = hasher.finalize();
    assert_eq!(result.len(), 32);
    for i in 0..32 {
        insert_value(
            &gadget_call.outputs[i],
            FieldElement::from_be_bytes_reduce(&[result[i]]),
            initial_witness,
        )?;
    }
    Ok(())
}
