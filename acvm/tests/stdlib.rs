mod solver;
use acir::{native_types::Witness, FieldElement};
use acvm::pwg::{ACVMStatus, ACVM};
use proptest::prelude::*;
use std::collections::BTreeMap;
use stdlib::blackbox_fallbacks::wu32::WU32;

use crate::solver::StubbedBackend;

proptest! {
    #[test]
    fn test_wu32_ror(x in 0..u32::MAX, y in 0..32_u32) {
        let fe = FieldElement::from(x as u128);
        let w = Witness(1);
        let result = x.rotate_right(y);
        let sha256_u32 = WU32::new(w);
        let (w, extra_gates, _) = sha256_u32.ror(y, 2);
        let witness_assignments = BTreeMap::from([(Witness(1), fe)]).into();
        let mut acvm = ACVM::new(StubbedBackend, extra_gates, witness_assignments);
        let solver_status = acvm.solve();

        prop_assert_eq!(acvm.witness_map().get(&w.inner).unwrap(), &FieldElement::from(result as u128));
        prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
    }

    #[test]
    fn test_wu32_euclidean_division(x in 0..u32::MAX, y in 0..u32::MAX) {
        let lhs = FieldElement::from(x as u128);
        let rhs = FieldElement::from(y as u128);
        let w1 = Witness(1);
        let w2 = Witness(2);
        let q = x.div_euclid(y);
        let r = x.rem_euclid(y);
        let u32_1 = WU32::new(w1);
        let u32_2 = WU32::new(w2);
        let (q_w, r_w, extra_gates, _) = WU32::euclidean_division(&u32_1, &u32_2, 3);
        let witness_assignments = BTreeMap::from([(Witness(1), lhs),(Witness(2), rhs)]).into();
        let mut acvm = ACVM::new(StubbedBackend, extra_gates, witness_assignments);
        let solver_status = acvm.solve();

        prop_assert_eq!(acvm.witness_map().get(&q_w.inner).unwrap(), &FieldElement::from(q as u128));
        prop_assert_eq!(acvm.witness_map().get(&r_w.inner).unwrap(), &FieldElement::from(r as u128));
        prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
    }

    #[test]
    fn test_wu32_add(x in 0..u32::MAX, y in 0..u32::MAX, z in 0..u32::MAX) {
        let lhs = FieldElement::from(x as u128);
        let rhs = FieldElement::from(y as u128);
        let rhs_z = FieldElement::from(z as u128);
        let result = FieldElement::from(((x as u128).wrapping_add(y as u128) % (1_u128 << 32)).wrapping_add(z as u128) % (1_u128 << 32));
        let w1 = Witness(1);
        let w2 = Witness(2);
        let w3 = Witness(3);
        let u32_1 = WU32::new(w1);
        let u32_2 = WU32::new(w2);
        let u32_3 = WU32::new(w3);
        let mut gates = Vec::new();
        let (w, extra_gates, num_witness) = u32_1.add(&u32_2, 4);
        gates.extend(extra_gates);
        let (w2, extra_gates, _) = w.add(&u32_3, num_witness);
        gates.extend(extra_gates);
        let witness_assignments = BTreeMap::from([(Witness(1), lhs), (Witness(2), rhs), (Witness(3), rhs_z)]).into();
        let mut acvm = ACVM::new(StubbedBackend, gates, witness_assignments);
        let solver_status = acvm.solve();

        prop_assert_eq!(acvm.witness_map().get(&w2.inner).unwrap(), &result);
        prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
    }

    #[test]
    fn test_wu32_sub(x in 0..u32::MAX, y in 0..u32::MAX, z in 0..u32::MAX) {
        let lhs = FieldElement::from(x as u128);
        let rhs = FieldElement::from(y as u128);
        let rhs_z = FieldElement::from(z as u128);
        let result = FieldElement::from(((x as u128).wrapping_sub(y as u128) % (1_u128 << 32)).wrapping_sub(z as u128) % (1_u128 << 32));
        let w1 = Witness(1);
        let w2 = Witness(2);
        let w3 = Witness(3);
        let u32_1 = WU32::new(w1);
        let u32_2 = WU32::new(w2);
        let u32_3 = WU32::new(w3);
        let mut gates = Vec::new();
        let (w, extra_gates, num_witness) = u32_1.sub(&u32_2, 4);
        gates.extend(extra_gates);
        let (w2, extra_gates, _) = w.sub(&u32_3, num_witness);
        gates.extend(extra_gates);
        let witness_assignments = BTreeMap::from([(Witness(1), lhs), (Witness(2), rhs), (Witness(3), rhs_z)]).into();
        let mut acvm = ACVM::new(StubbedBackend, gates, witness_assignments);
                let solver_status = acvm.solve();

        prop_assert_eq!(acvm.witness_map().get(&w2.inner).unwrap(), &result);
        prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
    }

    #[test]
    fn test_wu32_left_shift(x in 0..u32::MAX, y in 0..32_u32) {
        let lhs = FieldElement::from(x as u128);
        let w1 = Witness(1);
        let result = x.overflowing_shl(y).0;
        let u32_1 = WU32::new(w1);
        let (w, extra_gates, _) = u32_1.leftshift(y, 2);
        let witness_assignments = BTreeMap::from([(Witness(1), lhs)]).into();
        let mut acvm = ACVM::new(StubbedBackend, extra_gates, witness_assignments);
        let solver_status = acvm.solve();

        prop_assert_eq!(acvm.witness_map().get(&w.inner).unwrap(), &FieldElement::from(result as u128));
        prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
    }

    #[test]
    fn test_wu32_right_shift(x in 0..u32::MAX, y in 0..32_u32) {
        let lhs = FieldElement::from(x as u128);
        let w1 = Witness(1);
        let result = x.overflowing_shr(y).0;
        let u32_1 = WU32::new(w1);
        let (w, extra_gates, _) = u32_1.rightshift(y, 2);
        let witness_assignments = BTreeMap::from([(Witness(1), lhs)]).into();
        let mut acvm = ACVM::new(StubbedBackend, extra_gates, witness_assignments);
        let solver_status = acvm.solve();

        prop_assert_eq!(acvm.witness_map().get(&w.inner).unwrap(), &FieldElement::from(result as u128));
        prop_assert_eq!(solver_status, ACVMStatus::Solved, "should be fully solved");
    }
}
