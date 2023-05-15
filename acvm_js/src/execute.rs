use acvm::{
    acir::{
        circuit::{opcodes::BlackBoxFuncCall, Circuit},
        native_types::Witness,
        BlackBoxFunc,
    },
    pwg::{block::Blocks, hash, logic, range, signature},
    FieldElement, OpcodeResolution, OpcodeResolutionError, PartialWitnessGenerator,
    PartialWitnessGeneratorStatus,
};
use std::collections::BTreeMap;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{js_map_to_witness_map, witness_map_to_js_map};

struct SimulatedBackend;

impl PartialWitnessGenerator for SimulatedBackend {
    fn solve_black_box_function_call(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        func_call: &BlackBoxFuncCall,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        match func_call.name {
            BlackBoxFunc::SHA256 => hash::sha256(initial_witness, func_call),
            BlackBoxFunc::Blake2s => hash::blake2s(initial_witness, func_call),
            BlackBoxFunc::EcdsaSecp256k1 => {
                signature::ecdsa::secp256k1_prehashed(initial_witness, func_call)
            }
            BlackBoxFunc::AND | BlackBoxFunc::XOR => {
                logic::solve_logic_opcode(initial_witness, func_call)
            }
            BlackBoxFunc::RANGE => range::solve_range_opcode(initial_witness, func_call),
            BlackBoxFunc::HashToField128Security => {
                acvm::pwg::hash::hash_to_field_128_security(initial_witness, func_call)
            }
            BlackBoxFunc::Keccak256 => todo!("need to update to a newer version of ACVM"),
            BlackBoxFunc::AES
            | BlackBoxFunc::Pedersen
            | BlackBoxFunc::ComputeMerkleRoot
            | BlackBoxFunc::FixedBaseScalarMul
            | BlackBoxFunc::SchnorrVerify => {
                unimplemented!("opcode does not have a rust implementation")
            }
        }
    }
}

#[wasm_bindgen]
pub fn execute_circuit(circuit: Vec<u8>, initial_witness: js_sys::Map) -> js_sys::Map {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");
    let mut witness_map = js_map_to_witness_map(initial_witness);

    let mut blocks = Blocks::default();
    let solver_status = SimulatedBackend
        .solve(&mut witness_map, &mut blocks, circuit.opcodes)
        .expect("Threw error while executing circuit");
    assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved);

    witness_map_to_js_map(witness_map)
}
