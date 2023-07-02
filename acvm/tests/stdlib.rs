use std::collections::BTreeMap;

use acir::{
    circuit::{opcodes::FunctionInput},
    native_types::{Witness, WitnessMap},
    FieldElement,
};
use acvm::{
    pwg::{
        OpcodeResolution, OpcodeResolutionError,
        PartialWitnessGeneratorStatus, ACVM,
    },
    PartialWitnessGenerator,
};
use stdlib::custom_gate_fallbacks::sha256::Sha256U32;

struct StubbedPwg;

impl PartialWitnessGenerator for StubbedPwg {
    fn schnorr_verify(
        &self,
        _initial_witness: &mut WitnessMap,
        _public_key_x: FunctionInput,
        _public_key_y: FunctionInput,
        _signature_s: FunctionInput,
        _signature_e: FunctionInput,
        _message: &[FunctionInput],
        _output: Witness,
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        panic!("Path not trodden by this test")
    }

    fn pedersen(
        &self,
        _initial_witness: &mut WitnessMap,
        _inputs: &[FunctionInput],
        _domain_separator: u32,
        _outputs: (Witness, Witness),
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        panic!("Path not trodden by this test")
    }

    fn fixed_base_scalar_mul(
        &self,
        _initial_witness: &mut WitnessMap,
        _input: FunctionInput,
        _outputs: (Witness, Witness),
    ) -> Result<OpcodeResolution, OpcodeResolutionError> {
        panic!("Path not trodden by this test")
    }
}

#[test]
fn test_sha256_u32_ror() {
    let fe = FieldElement::from(0b10010_u128);
    let w = Witness(1);

    let sha256_u32 = Sha256U32::new(w);

    let (_, extra_gates, _) = sha256_u32.ror(3, 2);

    let witness_assignments: WitnessMap = BTreeMap::from([(Witness(1), fe)]).into();

    let mut acvm = ACVM::new(StubbedPwg, extra_gates, witness_assignments);
    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve().expect("should succeed");

    assert_eq!(acvm.witness_map().get(&Witness(2)).unwrap(), &FieldElement::from(1073741826_u128));
    assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved, "should be fully solved");
}
