use std::collections::BTreeMap;

use acir::{
    circuit::opcodes::{BlackBoxFuncCall, FunctionInput},
    native_types::Witness,
    FieldElement,
};

use crate::{OpcodeNotSolvable, OpcodeResolutionError, PartialWitnessGenerator};

use super::OpcodeResolution;

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
    let inputs = bb_func.get_inputs_vec();
    match bb_func {
        _ if !contains_all_inputs(initial_witness, &inputs) => {
            if let Some(unassigned_witness) = first_missing_assignment(initial_witness, &inputs) {
                Ok(OpcodeResolution::Stalled(OpcodeNotSolvable::MissingAssignment(
                    unassigned_witness.0,
                )))
            } else {
                // This only exists because Rust won't let us bind in a pattern guard.
                // See https://github.com/rust-lang/rust/issues/51114
                unreachable!("Only reachable if the blackbox is stalled")
            }
        }
        BlackBoxFuncCall::AES { inputs, outputs } => backend.aes(initial_witness, inputs, outputs),
        BlackBoxFuncCall::AND { lhs, rhs, output } => {
            backend.and(initial_witness, lhs, rhs, output)
        }
        BlackBoxFuncCall::XOR { lhs, rhs, output } => {
            backend.xor(initial_witness, lhs, rhs, output)
        }
        BlackBoxFuncCall::RANGE { input } => backend.range(initial_witness, input),
        BlackBoxFuncCall::SHA256 { inputs, outputs } => {
            backend.sha256(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall::Blake2s { inputs, outputs } => {
            backend.blake2s(initial_witness, inputs, outputs)
        }
        BlackBoxFuncCall::ComputeMerkleRoot { inputs, output } => {
            backend.compute_merkle_root(initial_witness, inputs, output)
        }
        BlackBoxFuncCall::SchnorrVerify { inputs, output } => {
            backend.schnorr_verify(initial_witness, inputs, output)
        }
        BlackBoxFuncCall::Pedersen { hash_index, inputs, outputs } => {
            backend.pedersen(initial_witness, *hash_index, inputs, outputs)
        }
        BlackBoxFuncCall::HashToField128Security { inputs, output } => {
            backend.hash_to_field128_security(initial_witness, inputs, output)
        }
        BlackBoxFuncCall::EcdsaSecp256k1 { inputs, output } => {
            backend.ecdsa_secp256k1(initial_witness, inputs, output)
        }
        BlackBoxFuncCall::FixedBaseScalarMul { input, outputs } => {
            backend.fixed_base_scalar_mul(initial_witness, input, outputs)
        }
        BlackBoxFuncCall::Keccak256 { inputs, outputs } => {
            backend.keccak256(initial_witness, inputs, outputs)
        }
    }
}
