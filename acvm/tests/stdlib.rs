use std::collections::BTreeMap;

use acir::{
    native_types::{Witness, WitnessMap},
    FieldElement,
};
use acvm::{
    pwg::{OpcodeResolutionError, PartialWitnessGeneratorStatus, ACVM},
    BlackBoxFunctionSolver,
};
use stdlib::custom_gate_fallbacks::sha256::Sha256U32;

struct StubbedPwg;

impl BlackBoxFunctionSolver for StubbedPwg {
    fn schnorr_verify(
        &self,
        _public_key_x: &FieldElement,
        _public_key_y: &FieldElement,
        _signature_s: &FieldElement,
        _signature_e: &FieldElement,
        _message: &[u8],
    ) -> Result<bool, OpcodeResolutionError> {
        panic!("Path not trodden by this test")
    }
    fn pedersen(
        &self,
        _inputs: &[FieldElement],
        _domain_separator: u32,
    ) -> Result<(FieldElement, FieldElement), OpcodeResolutionError> {
        panic!("Path not trodden by this test")
    }
    fn fixed_base_scalar_mul(
        &self,
        _input: &FieldElement,
    ) -> Result<(FieldElement, FieldElement), OpcodeResolutionError> {
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
