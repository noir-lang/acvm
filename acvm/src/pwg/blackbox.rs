use std::collections::BTreeMap;

use acir::{
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput},
    native_types::Witness,
    BlackBoxFunc, FieldElement,
};

use crate::{OpcodeNotSolvable, OpcodeResolutionError, PartialWitnessGenerator};

use super::{
    hash::{blake2s, hash_to_field_128_security, keccak256, sha256},
    logic::solve_logic_opcode,
    range::solve_range_opcode,
    signature::ecdsa::secp256k1_prehashed,
    OpcodeResolution,
};

/// Check if all of the inputs to the function have assignments
///
/// Returns the first missing assignment if any are missing
fn first_missing_assignment(
    witness_assignments: &BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
) -> Option<Witness> {
    inputs.iter().find_map(|input| {
        if witness_assignments.contains_key(&input.witness) {
            None
        } else {
            Some(input.witness)
        }
    })
}

/// Check if all of the inputs to the function have assignments
fn contains_all_inputs(
    witness_assignments: &BTreeMap<Witness, FieldElement>,
    inputs: &[FunctionInput],
) -> bool {
    inputs.iter().all(|input| witness_assignments.contains_key(&input.witness))
}

pub(crate) fn solve(
    backend: &impl PartialWitnessGenerator,
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    bb_func: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    if !contains_all_inputs(initial_witness, &bb_func.inputs) {
        if let Some(unassigned_witness) = first_missing_assignment(initial_witness, &bb_func.inputs)
        {
            return Ok(OpcodeResolution::Stalled(OpcodeNotSolvable::MissingAssignment(
                unassigned_witness.0,
            )));
        }
    }
    match bb_func.name {
        BlackBoxFunc::AND | BlackBoxFunc::XOR => solve_logic_opcode(initial_witness, bb_func),
        BlackBoxFunc::RANGE => solve_range_opcode(initial_witness, bb_func),
        BlackBoxFunc::SHA256 => sha256(initial_witness, bb_func),
        BlackBoxFunc::Blake2s => blake2s(initial_witness, bb_func),
        BlackBoxFunc::Keccak256 => keccak256(initial_witness, bb_func),
        BlackBoxFunc::ComputeMerkleRoot => {
            backend.compute_merkle_root(initial_witness, &bb_func.inputs, &bb_func.outputs)
        }
        BlackBoxFunc::SchnorrVerify => {
            backend.schnorr_verify(initial_witness, &bb_func.inputs, &bb_func.outputs)
        }
        BlackBoxFunc::Pedersen => {
            backend.pedersen(initial_witness, &bb_func.inputs, &bb_func.outputs)
        }
        BlackBoxFunc::HashToField128Security => {
            hash_to_field_128_security(initial_witness, bb_func)
        }
        BlackBoxFunc::EcdsaSecp256k1 => secp256k1_prehashed(initial_witness, bb_func),
        BlackBoxFunc::FixedBaseScalarMul => {
            backend.fixed_base_scalar_mul(initial_witness, &bb_func.inputs, &bb_func.outputs)
        }
        BlackBoxFunc::AES => backend.aes(initial_witness, &bb_func.inputs, &bb_func.outputs),
    }
}
