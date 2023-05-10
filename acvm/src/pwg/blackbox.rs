use acir::{
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput},
    native_types::{Witness, WitnessMap},
    BlackBoxFunc,
};

use crate::{OpcodeNotSolvable, OpcodeResolutionError, PartialWitnessGenerator};

use super::OpcodeResolution;

/// Check if all of the inputs to the function have assignments
///
/// Returns the first missing assignment if any are missing
fn first_missing_assignment(
    witness_assignments: &WitnessMap,
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
fn contains_all_inputs(witness_assignments: &WitnessMap, inputs: &[FunctionInput]) -> bool {
    inputs.iter().all(|input| witness_assignments.contains_key(&input.witness))
}

pub(crate) fn solve(
    backend: &impl PartialWitnessGenerator,
    initial_witness: &mut WitnessMap,
    bb_func: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {
    match bb_func {
        BlackBoxFuncCall { inputs, .. } if !contains_all_inputs(initial_witness, inputs) => {
            if let Some(unassigned_witness) = first_missing_assignment(initial_witness, inputs) {
                Ok(OpcodeResolution::Stalled(OpcodeNotSolvable::MissingAssignment(
                    unassigned_witness.0,
                )))
            } else {
                // This only exists because Rust won't let us bind in a pattern guard.
                // See https://github.com/rust-lang/rust/issues/51114
                unreachable!("Only reachable if the blackbox is stalled")
            }
        }
        BlackBoxFuncCall { name: BlackBoxFunc::AES, inputs, outputs } => {
            backend.aes(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::AND, inputs, outputs } => {
            backend.and(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::XOR, inputs, outputs } => {
            backend.xor(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::RANGE, inputs, outputs } => {
            assert!(outputs.is_empty());
            backend.range(initial_witness, inputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::SHA256, inputs, outputs } => {
            backend.sha256(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::Blake2s, inputs, outputs } => {
            backend.blake2s(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::ComputeMerkleRoot, inputs, outputs } => {
            backend.compute_merkle_root(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::SchnorrVerify, inputs, outputs } => {
            backend.schnorr_verify(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::Pedersen, inputs, outputs } => {
            backend.pedersen(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::HashToField128Security, inputs, outputs } => {
            backend.hash_to_field_128_security(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::EcdsaSecp256k1, inputs, outputs } => {
            backend.ecdsa_secp256k1(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::FixedBaseScalarMul, inputs, outputs } => {
            backend.fixed_base_scalar_mul(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall { name: BlackBoxFunc::Keccak256, inputs, outputs } => {
            backend.keccak256(initial_witness, inputs, outputs)
        }
    }
}
