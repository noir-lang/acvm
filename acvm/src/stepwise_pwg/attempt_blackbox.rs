use std::collections::BTreeMap;

use acir::{circuit::opcodes::BlackBoxFuncCall, native_types::Witness, BlackBoxFunc, FieldElement};

use crate::{pwg, OpcodeResolutionError};

pub enum AttemptBlackBoxOutcome {
    Solved,
    Skipped,
    Blocked,
}

pub fn attempt_black_box(
    witness_skeleton: &mut BTreeMap<Witness, FieldElement>,
    bb_func_call: &BlackBoxFuncCall,
) -> Result<AttemptBlackBoxOutcome, OpcodeResolutionError> {
    let outcome = match bb_func_call.name {
        BlackBoxFunc::SHA256 => {
            pwg::hash::sha256(witness_skeleton, bb_func_call);
            AttemptBlackBoxOutcome::Solved
        }
        BlackBoxFunc::Blake2s => {
            pwg::hash::blake2s(witness_skeleton, bb_func_call);
            AttemptBlackBoxOutcome::Solved
        }
        BlackBoxFunc::EcdsaSecp256k1 => {
            match pwg::signature::ecdsa::secp256k1_prehashed(witness_skeleton, &bb_func_call) {
                Ok(_) => AttemptBlackBoxOutcome::Solved,
                Err(err) => return Err(err),
            }
        }
        BlackBoxFunc::AND | BlackBoxFunc::XOR => {
            match pwg::logic::solve_logic_opcode(witness_skeleton, bb_func_call) {
                Ok(_) => AttemptBlackBoxOutcome::Solved,
                Err(err) => return Err(err),
            }
        }
        BlackBoxFunc::RANGE => {
            match pwg::range::solve_range_opcode(witness_skeleton, bb_func_call) {
                Ok(_) => AttemptBlackBoxOutcome::Solved,
                Err(err) => return Err(err),
            }
        }
        BlackBoxFunc::AES
        | BlackBoxFunc::MerkleMembership
        | BlackBoxFunc::SchnorrVerify
        | BlackBoxFunc::Pedersen
        | BlackBoxFunc::HashToField128Security
        | BlackBoxFunc::FixedBaseScalarMul => {
            // TODO: Which of the above can also be solved deterministicly
            if inputs_are_ready(witness_skeleton, bb_func_call) {
                AttemptBlackBoxOutcome::Blocked
            } else {
                AttemptBlackBoxOutcome::Skipped
            }
        }
    };
    Ok(outcome)
}

fn inputs_are_ready(
    witness_skeleton: &mut BTreeMap<Witness, FieldElement>,
    bb_func_call: &BlackBoxFuncCall,
) -> bool {
    bb_func_call
        .inputs
        .iter()
        .all(|input| witness_skeleton.contains_key(&input.witness))
}
