use acir::{circuit::opcodes::BlackBoxFuncCall, native_types::Witness, FieldElement};
use blake2::{Blake2s, Digest};
use sha2::Sha256;
use std::collections::BTreeMap;

use crate::{OpcodeResolution, OpcodeResolutionError};

use super::witness_to_value;

pub fn blake2s(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    gadget_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    generic_hash_256::<Blake2s>(initial_witness, gadget_call)
}

pub fn sha256(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    gadget_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    generic_hash_256::<Sha256>(initial_witness, gadget_call)
}

fn generic_hash_256<D: Digest>(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    gadget_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    let mut hasher = D::new();

    // Read witness assignments into hasher.
    for input in gadget_call.inputs.iter() {
        let witness = input.witness;
        let num_bits = input.num_bits as usize;

        let witness_assignment = witness_to_value(initial_witness, witness)?;
        let bytes = witness_assignment.fetch_nearest_bytes(num_bits);
        hasher.update(bytes);
    }

    // Perform hash and write outputs to witness map.
    let result = hasher.finalize();
    for i in 0..32 {
        initial_witness
            .insert(gadget_call.outputs[i], FieldElement::from_be_bytes_reduce(&[result[i]]));
    }

    Ok(OpcodeResolution::Solved)
}
