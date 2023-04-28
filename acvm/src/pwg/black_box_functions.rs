use std::collections::BTreeMap;

use acir::{native_types::Witness, FieldElement, circuit::opcodes::BlackBoxFuncCall, BlackBoxFunc};

use crate::{OpcodeResolutionError, OpcodeResolution, OpcodeNotSolvable};

use super::hash;



pub fn solve_black_box_function(
    initial_witness: &mut BTreeMap<Witness, FieldElement>,
    func_call: &BlackBoxFuncCall,
) -> Result<OpcodeResolution, OpcodeResolutionError> {

    match func_call.name {
        BlackBoxFunc::AES 
        | BlackBoxFunc::AND
        | BlackBoxFunc::XOR
        | BlackBoxFunc::RANGE
        | BlackBoxFunc::SHA256
        | BlackBoxFunc::Blake2s
        | BlackBoxFunc::MerkleMembership
        | BlackBoxFunc::SchnorrVerify
        | BlackBoxFunc::Pedersen
        | BlackBoxFunc::HashToField128Security
        | BlackBoxFunc::EcdsaSecp256k1
        | BlackBoxFunc::FixedBaseScalarMul => Err(OpcodeResolutionError::OpcodeNotSolvable(OpcodeNotSolvable::MissingAssignment(0))),
        //self.solve_black_box_function_call(initial_witness, func_call),
        BlackBoxFunc::Keccak256 => {
            hash::keccak256(initial_witness, func_call)?;
            Ok(OpcodeResolution::Solved)
        },
    }
}
