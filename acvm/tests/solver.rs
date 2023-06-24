use std::collections::BTreeMap;

use acir::{
    brillig_vm::{self, BinaryFieldOp, RegisterIndex, RegisterOrMemory, Value},
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        directives::Directive,
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

use acvm::{
    pwg::{ForeignCallWaitInfo, OpcodeResolutionError, PartialWitnessGeneratorStatus, ACVM},
    BlackBoxFunctionSolver,
};

struct StubbedBackend;

impl BlackBoxFunctionSolver for StubbedBackend {
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
fn inversion_brillig_oracle_equivalence() {
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     let z = x + y;
    //     assert( 1/z == Oracle("inverse", x + y) );
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_z = Witness(4);
    let w_z_inverse = Witness(5);
    let w_x_plus_y = Witness(6);
    let w_equal_res = Witness(7);

    let equal_opcode = brillig_vm::Opcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: RegisterIndex::from(0),
        rhs: RegisterIndex::from(1),
        destination: RegisterIndex::from(2),
    };

    let brillig_data = Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                // Input Register 0
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                q_c: fe_0,
            }),
            BrilligInputs::Single(Expression::default()), // Input Register 1
        ],
        // This tells the BrilligSolver which witnesses its output registers correspond to
        outputs: vec![
            BrilligOutputs::Simple(w_x_plus_y), // Output Register 0 - from input
            BrilligOutputs::Simple(w_oracle),   // Output Register 1
            BrilligOutputs::Simple(w_equal_res), // Output Register 2
        ],
        // stack of foreign call/oracle resolutions, starts empty
        foreign_call_results: vec![],
        bytecode: vec![
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            brillig_vm::Opcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
            },
        ],
        predicate: None,
    };

    let opcodes = vec![
        Opcode::Brillig(brillig_data),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
            q_c: fe_0,
        }),
        Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![(fe_1, w_z, w_z_inverse)],
            linear_combinations: vec![],
            q_c: -fe_1,
        }),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
            q_c: fe_0,
        }),
    ];

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
    ])
    .into();

    let mut acvm = ACVM::new(StubbedBackend, opcodes, witness_assignments);
    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve().expect("should stall on brillig call");

    assert_eq!(
        solver_status,
        PartialWitnessGeneratorStatus::RequiresForeignCall,
        "Should require oracle data"
    );
    assert!(acvm.unresolved_opcodes().is_empty(), "brillig should have been removed");

    let foreign_call_wait_info: &ForeignCallWaitInfo =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    // As caller of VM, need to resolve foreign calls
    let foreign_call_result = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());
    // Alter Brillig oracle opcode with foreign call resolution
    acvm.resolve_pending_foreign_call(foreign_call_result.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve().expect("should not stall on brillig call");
    assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved, "should be fully solved");
}

#[test]
fn double_inversion_brillig_oracle() {
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     let z = x + y;
    //     let ij = i + j;
    //     assert( 1/z == Oracle("inverse", x + y) );
    //     assert( 1/ij == Oracle("inverse", i + j) );
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_z = Witness(4);
    let w_z_inverse = Witness(5);
    let w_x_plus_y = Witness(6);
    let w_equal_res = Witness(7);
    let w_i = Witness(8);
    let w_j = Witness(9);
    let w_ij_oracle = Witness(10);
    let w_i_plus_j = Witness(11);

    let equal_opcode = brillig_vm::Opcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: RegisterIndex::from(0),
        rhs: RegisterIndex::from(1),
        destination: RegisterIndex::from(4),
    };

    let brillig_data = Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                // Input Register 0
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                q_c: fe_0,
            }),
            BrilligInputs::Single(Expression::default()), // Input Register 1
            BrilligInputs::Single(Expression {
                // Input Register 2
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_i), (fe_1, w_j)],
                q_c: fe_0,
            }),
        ],
        outputs: vec![
            BrilligOutputs::Simple(w_x_plus_y), // Output Register 0 - from input
            BrilligOutputs::Simple(w_oracle),   // Output Register 1
            BrilligOutputs::Simple(w_i_plus_j), // Output Register 2 - from input
            BrilligOutputs::Simple(w_ij_oracle), // Output Register 3
            BrilligOutputs::Simple(w_equal_res), // Output Register 4
        ],
        // stack of foreign call/oracle resolutions, starts empty
        foreign_call_results: vec![],
        bytecode: vec![
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            brillig_vm::Opcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
            },
            brillig_vm::Opcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(3))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(2))],
            },
        ],
        predicate: None,
    };

    let opcodes = vec![
        Opcode::Brillig(brillig_data),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
            q_c: fe_0,
        }),
        Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![(fe_1, w_z, w_z_inverse)],
            linear_combinations: vec![],
            q_c: -fe_1,
        }),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
            q_c: fe_0,
        }),
    ];

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
        (Witness(8), FieldElement::from(5u128)),
        (Witness(9), FieldElement::from(10u128)),
    ])
    .into();

    let mut acvm = ACVM::new(StubbedBackend, opcodes, witness_assignments);

    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve().expect("should stall on oracle");
    assert_eq!(
        solver_status,
        PartialWitnessGeneratorStatus::RequiresForeignCall,
        "Should require oracle data"
    );
    assert!(acvm.unresolved_opcodes().is_empty(), "brillig should have been removed");

    let foreign_call_wait_info: &ForeignCallWaitInfo =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    let x_plus_y_inverse = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());

    // Resolve Brillig foreign call
    acvm.resolve_pending_foreign_call(x_plus_y_inverse.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve().expect("should stall on brillig call");
    assert_eq!(
        solver_status,
        PartialWitnessGeneratorStatus::RequiresForeignCall,
        "Should require oracle data"
    );
    assert!(acvm.unresolved_opcodes().is_empty(), "should be fully solved");

    let foreign_call_wait_info =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    let i_plus_j_inverse = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());
    assert_ne!(x_plus_y_inverse, i_plus_j_inverse);

    // Alter Brillig oracle opcode
    acvm.resolve_pending_foreign_call(i_plus_j_inverse.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve().expect("should not stall on brillig call");
    assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved, "should be fully solved");
}

#[test]
fn oracle_dependent_execution() {
    // This test ensures that we properly track the list of opcodes which still need to be resolved
    // across any brillig foreign calls we may have to perform.
    //
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field) {
    //     assert(x == y);
    //     let x_inv = Oracle("inverse", x);
    //     let y_inv = Oracle("inverse", y);
    //
    //     assert(x_inv == y_inv);
    // }
    // Also performs an unrelated equality check
    // just for the sake of testing multiple brillig opcodes.
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_x_inv = Witness(3);
    let w_y_inv = Witness(4);

    let brillig_data = Brillig {
        inputs: vec![
            BrilligInputs::Single(w_x.into()),            // Input Register 0
            BrilligInputs::Single(Expression::default()), // Input Register 1
            BrilligInputs::Single(w_y.into()),            // Input Register 2,
        ],
        outputs: vec![
            BrilligOutputs::Simple(w_x),     // Output Register 0 - from input
            BrilligOutputs::Simple(w_y_inv), // Output Register 1
            BrilligOutputs::Simple(w_y),     // Output Register 2 - from input
            BrilligOutputs::Simple(w_y_inv), // Output Register 3
        ],
        // stack of foreign call/oracle resolutions, starts empty
        foreign_call_results: vec![],
        bytecode: vec![
            // Oracles are named 'foreign calls' in brillig
            brillig_vm::Opcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
            },
            brillig_vm::Opcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(3))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(2))],
            },
        ],
        predicate: None,
    };

    // This equality check can be executed immediately before resolving any foreign calls.
    let equality_check = Expression {
        mul_terms: vec![],
        linear_combinations: vec![(-fe_1, w_x), (fe_1, w_y)],
        q_c: fe_0,
    };

    // This equality check relies on the outputs of the Brillig call.
    // It then cannot be solved until the foreign calls are resolved.
    let inverse_equality_check = Expression {
        mul_terms: vec![],
        linear_combinations: vec![(-fe_1, w_x_inv), (fe_1, w_y_inv)],
        q_c: fe_0,
    };

    let opcodes = vec![
        Opcode::Arithmetic(equality_check),
        Opcode::Brillig(brillig_data),
        Opcode::Arithmetic(inverse_equality_check.clone()),
    ];

    let witness_assignments =
        BTreeMap::from([(w_x, FieldElement::from(2u128)), (w_y, FieldElement::from(2u128))]).into();

    let mut acvm = ACVM::new(StubbedBackend, opcodes, witness_assignments);

    // use the partial witness generation solver with our acir program
    let solver_status = acvm.solve().expect("should stall on oracle");
    assert_eq!(
        solver_status,
        PartialWitnessGeneratorStatus::RequiresForeignCall,
        "Should require oracle data"
    );
    assert_eq!(acvm.unresolved_opcodes().len(), 1, "brillig should have been removed");
    assert_eq!(
        acvm.unresolved_opcodes()[0],
        Opcode::Arithmetic(inverse_equality_check.clone()),
        "Equality check of inverses should still be waiting to be resolved"
    );

    let foreign_call_wait_info: &ForeignCallWaitInfo =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    // Resolve Brillig foreign call
    let x_inverse = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());
    acvm.resolve_pending_foreign_call(x_inverse.into());

    // After filling data request, continue solving
    let solver_status = acvm.solve().expect("should stall on oracle");
    assert_eq!(
        solver_status,
        PartialWitnessGeneratorStatus::RequiresForeignCall,
        "Should require oracle data"
    );
    assert_eq!(acvm.unresolved_opcodes().len(), 1, "brillig should have been removed");
    assert_eq!(
        acvm.unresolved_opcodes()[0],
        Opcode::Arithmetic(inverse_equality_check),
        "Equality check of inverses should still be waiting to be resolved"
    );

    let foreign_call_wait_info: &ForeignCallWaitInfo =
        acvm.get_pending_foreign_call().expect("should have a brillig foreign call request");
    assert_eq!(foreign_call_wait_info.inputs.len(), 1, "Should be waiting for a single input");

    // Resolve Brillig foreign call
    let y_inverse = Value::from(foreign_call_wait_info.inputs[0][0].to_field().inverse());
    acvm.resolve_pending_foreign_call(y_inverse.into());

    // We've resolved all the brillig foreign calls so we should be able to complete execution now.

    // After filling data request, continue solving
    let solver_status = acvm.solve().expect("should not stall on brillig call");
    assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved, "should be fully solved");
}

#[test]
fn brillig_oracle_predicate() {
    // Opcodes below describe the following:
    // fn main(x : Field, y : pub Field, cond: bool) {
    //     let z = x + y;
    //     let z_inverse = 1/z
    //     if cond {
    //         assert( z_inverse == Oracle("inverse", x + y) );
    //     }
    // }
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_z = Witness(4);
    let w_z_inverse = Witness(5);
    let w_x_plus_y = Witness(6);
    let w_equal_res = Witness(7);
    let w_lt_res = Witness(8);

    let equal_opcode = brillig_vm::Opcode::BinaryFieldOp {
        op: BinaryFieldOp::Equals,
        lhs: RegisterIndex::from(0),
        rhs: RegisterIndex::from(1),
        destination: RegisterIndex::from(2),
    };

    let brillig_opcode = Opcode::Brillig(Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
                q_c: fe_0,
            }),
            BrilligInputs::Single(Expression::default()),
        ],
        outputs: vec![
            BrilligOutputs::Simple(w_x_plus_y),
            BrilligOutputs::Simple(w_oracle),
            BrilligOutputs::Simple(w_equal_res),
            BrilligOutputs::Simple(w_lt_res),
        ],
        bytecode: vec![
            equal_opcode,
            // Oracles are named 'foreign calls' in brillig
            brillig_vm::Opcode::ForeignCall {
                function: "invert".into(),
                destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
                inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
            },
        ],
        predicate: Some(Expression::default()),
        // oracle results
        foreign_call_results: vec![],
    });

    let opcodes = vec![
        brillig_opcode,
        Opcode::Arithmetic(Expression {
            mul_terms: vec![],
            linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
            q_c: fe_0,
        }),
        Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
        Opcode::Arithmetic(Expression {
            mul_terms: vec![(fe_1, w_z, w_z_inverse)],
            linear_combinations: vec![],
            q_c: -fe_1,
        }),
    ];

    let witness_assignments = BTreeMap::from([
        (Witness(1), FieldElement::from(2u128)),
        (Witness(2), FieldElement::from(3u128)),
    ])
    .into();

    let mut acvm = ACVM::new(StubbedBackend, opcodes, witness_assignments);
    let solver_status = acvm.solve().expect("should not stall on brillig call");
    assert_eq!(solver_status, PartialWitnessGeneratorStatus::Solved, "should be fully solved");
}
