//! This integration test defines a set of circuits which are used in order to test the acvm_js package.
//!
//! The acvm_js test suite contains serialized [circuits][`Circuit`] which must be kept in sync with the format
//! outputted from the [ACIR crate][acir].
//! Breaking changes to the serialization format then require refreshing acvm_js's test suite.
//! This file contains Rust definitions of these circuits and outputs the updated serialized format.
//!
//! These tests also check this circuit serialization against an expected value, erroring if the serialization changes.
//! Generally in this situation we just need to refresh the `expected_serialization` variables to match the
//! actual output, **HOWEVER** note that this results in a breaking change to the ACIR format.

use std::collections::BTreeSet;

use acir::{
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        directives::Directive,
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Circuit, Opcode, PublicInputs,
    },
    native_types::{Expression, Witness},
};
use acir_field::FieldElement;
use brillig::{BinaryFieldOp, HeapArray, RegisterIndex, RegisterOrMemory};

#[test]
fn addition_circuit() {
    let addition = Opcode::Arithmetic(Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![
            (FieldElement::one(), Witness(1)),
            (FieldElement::one(), Witness(2)),
            (-FieldElement::one(), Witness(3)),
        ],
        q_c: FieldElement::zero(),
    });

    let circuit = Circuit {
        current_witness_index: 4,
        opcodes: vec![addition],
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        return_values: PublicInputs([Witness(3)].into()),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 187, 13, 192, 32, 12, 68, 249, 100, 32, 27,
        219, 96, 119, 89, 37, 40, 176, 255, 8, 17, 18, 5, 74, 202, 240, 154, 235, 158, 238, 238,
        112, 206, 121, 247, 37, 206, 60, 103, 194, 63, 208, 111, 116, 133, 197, 69, 144, 153, 91,
        73, 13, 9, 47, 72, 86, 85, 128, 165, 102, 69, 69, 81, 185, 147, 18, 53, 101, 45, 86, 173,
        128, 33, 83, 195, 46, 70, 125, 202, 226, 190, 94, 16, 166, 103, 108, 13, 203, 151, 254,
        245, 233, 224, 1, 1, 52, 166, 127, 120, 1, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn fixed_base_scalar_mul_circuit() {
    let fixed_base_scalar_mul = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::FixedBaseScalarMul {
        input: FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() },
        outputs: (Witness(2), Witness(3)),
    });

    let circuit = Circuit {
        current_witness_index: 4,
        opcodes: vec![fixed_base_scalar_mul],
        private_parameters: BTreeSet::from([Witness(1)]),
        return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 137, 91, 10, 0, 0, 4, 4, 215, 227, 203, 253, 207,
        43, 132, 146, 169, 105, 106, 87, 1, 16, 154, 170, 77, 61, 229, 84, 222, 191, 240, 169, 156,
        61, 0, 36, 111, 164, 5, 80, 0, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn pedersen_circuit() {
    let pedersen = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Pedersen {
        inputs: vec![FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() }],
        outputs: (Witness(2), Witness(3)),
        domain_separator: 0,
    });

    let circuit = Circuit {
        current_witness_index: 4,
        opcodes: vec![pedersen],
        private_parameters: BTreeSet::from([Witness(1)]),
        return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 138, 9, 10, 0, 64, 8, 2, 103, 15, 250, 255, 139,
        163, 162, 130, 72, 16, 149, 241, 3, 135, 84, 164, 172, 173, 213, 175, 251, 45, 198, 96,
        243, 211, 50, 152, 67, 220, 211, 92, 0, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn schnorr_verify_circuit() {
    let public_key_x =
        FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() };
    let public_key_y =
        FunctionInput { witness: Witness(2), num_bits: FieldElement::max_num_bits() };
    let signature =
        (3..(3 + 64)).map(|i| FunctionInput { witness: Witness(i), num_bits: 8 }).collect();
    let message = ((3 + 64)..(3 + 64 + 10))
        .map(|i| FunctionInput { witness: Witness(i), num_bits: 8 })
        .collect();
    let output = Witness(3 + 64 + 10);
    let last_input = output.witness_index() - 1;

    let schnorr = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::SchnorrVerify {
        public_key_x,
        public_key_y,
        signature,
        message,
        output,
    });

    let circuit = Circuit {
        current_witness_index: 100,
        opcodes: vec![schnorr],
        private_parameters: BTreeSet::from_iter((1..=last_input).map(Witness)),
        return_values: PublicInputs(BTreeSet::from([output])),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 210, 87, 78, 2, 1, 20, 134, 209, 177, 247, 222, 123,
        71, 68, 68, 68, 68, 68, 68, 68, 68, 68, 221, 133, 251, 95, 130, 145, 27, 206, 36, 78, 50,
        57, 16, 94, 200, 253, 191, 159, 36, 73, 134, 146, 193, 19, 142, 241, 183, 255, 14, 179,
        233, 247, 145, 254, 59, 217, 127, 71, 57, 198, 113, 78, 48, 125, 167, 56, 205, 25, 206,
        114, 142, 243, 92, 224, 34, 151, 184, 204, 21, 174, 114, 141, 235, 220, 224, 38, 183, 184,
        205, 29, 238, 114, 143, 251, 60, 224, 33, 143, 120, 204, 19, 158, 242, 140, 25, 158, 51,
        203, 11, 230, 120, 201, 60, 175, 88, 224, 53, 139, 188, 97, 137, 183, 44, 243, 142, 21,
        222, 179, 202, 7, 214, 248, 200, 58, 159, 216, 224, 51, 155, 124, 97, 235, 223, 142, 241,
        188, 250, 222, 230, 27, 59, 124, 103, 151, 31, 236, 241, 147, 95, 252, 246, 57, 158, 104,
        47, 186, 139, 214, 162, 179, 104, 44, 250, 74, 219, 154, 242, 63, 162, 165, 232, 40, 26,
        138, 126, 162, 157, 232, 38, 154, 137, 94, 162, 149, 232, 36, 26, 137, 62, 162, 141, 232,
        34, 154, 136, 30, 162, 133, 232, 32, 26, 136, 253, 99, 251, 195, 100, 176, 121, 236, 29,
        91, 159, 218, 56, 99, 219, 172, 77, 115, 182, 204, 219, 176, 96, 187, 162, 205, 74, 182,
        42, 219, 168, 98, 155, 170, 77, 106, 182, 168, 219, 160, 225, 246, 77, 55, 111, 185, 113,
        219, 109, 59, 110, 218, 117, 203, 158, 27, 166, 55, 75, 239, 150, 184, 101, 250, 252, 1,
        19, 89, 159, 101, 220, 3, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn simple_brillig_foreign_call() {
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let w_x = Witness(1);
    let w_y = Witness(2);
    let w_oracle = Witness(3);
    let w_z = Witness(4);
    let w_z_inverse = Witness(5);
    let w_x_plus_y = Witness(6);
    let w_equal_res = Witness(7);

    let equal_opcode = brillig::Opcode::BinaryFieldOp {
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
            brillig::Opcode::ForeignCall {
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
    let circuit = Circuit {
        current_witness_index: 8,
        opcodes,
        private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 148, 209, 10, 195, 32, 12, 69, 99, 109, 183, 126,
        78, 82, 181, 141, 111, 251, 149, 201, 44, 236, 101, 15, 99, 236, 251, 183, 49, 11, 161,
        245, 173, 233, 5, 137, 4, 57, 120, 111, 208, 30, 0, 58, 248, 203, 126, 87, 3, 91, 45, 189,
        75, 169, 184, 79, 100, 20, 89, 141, 30, 11, 43, 214, 213, 216, 86, 48, 79, 34, 239, 159,
        206, 149, 172, 229, 190, 21, 61, 83, 106, 47, 56, 247, 199, 59, 63, 95, 166, 114, 74, 246,
        170, 178, 186, 54, 15, 27, 173, 195, 209, 251, 60, 13, 153, 28, 93, 113, 136, 137, 3, 250,
        144, 70, 38, 166, 192, 225, 54, 176, 115, 153, 61, 79, 49, 197, 9, 35, 121, 151, 105, 14,
        209, 205, 5, 214, 234, 221, 11, 229, 88, 186, 85, 208, 90, 222, 37, 27, 20, 115, 16, 200,
        205, 179, 222, 203, 182, 138, 254, 187, 3, 230, 101, 160, 254, 189, 45, 250, 0, 87, 206,
        28, 176, 11, 5, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}

#[test]
fn complex_brillig_foreign_call() {
    let fe_0 = FieldElement::zero();
    let fe_1 = FieldElement::one();
    let a = Witness(1);
    let b = Witness(2);
    let c = Witness(3);

    let a_times_2 = Witness(4);
    let b_times_3 = Witness(5);
    let c_times_4 = Witness(6);
    let a_plus_b_plus_c = Witness(7);
    let a_plus_b_plus_c_times_2 = Witness(8);

    let brillig_data = Brillig {
        inputs: vec![
            // Input Register 0
            BrilligInputs::Array(vec![
                Expression { mul_terms: vec![], linear_combinations: vec![(fe_1, a)], q_c: fe_0 },
                Expression { mul_terms: vec![], linear_combinations: vec![(fe_1, b)], q_c: fe_0 },
                Expression { mul_terms: vec![], linear_combinations: vec![(fe_1, c)], q_c: fe_0 },
            ]),
            // Input Register 1
            BrilligInputs::Single(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(fe_1, a), (fe_1, b), (fe_1, c)],
                q_c: fe_0,
            }),
        ],
        // This tells the BrilligSolver which witnesses its output registers correspond to
        outputs: vec![
            BrilligOutputs::Array(vec![a_times_2, b_times_3, c_times_4]), // Output Register 0
            BrilligOutputs::Simple(a_plus_b_plus_c),                      // Output Register 1
            BrilligOutputs::Simple(a_plus_b_plus_c_times_2),              // Output Register 2
        ],
        // stack of foreign call/oracle resolutions, starts empty
        foreign_call_results: vec![],
        bytecode: vec![
            // Oracles are named 'foreign calls' in brillig
            brillig::Opcode::ForeignCall {
                function: "complex".into(),
                inputs: vec![
                    RegisterOrMemory::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
                    RegisterOrMemory::RegisterIndex(RegisterIndex::from(1)),
                ],
                destinations: vec![
                    RegisterOrMemory::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
                    RegisterOrMemory::RegisterIndex(RegisterIndex::from(1)),
                    RegisterOrMemory::RegisterIndex(RegisterIndex::from(2)),
                ],
            },
        ],
        predicate: None,
    };

    let opcodes = vec![Opcode::Brillig(brillig_data)];
    let circuit = Circuit {
        current_witness_index: 8,
        opcodes,
        private_parameters: BTreeSet::from([Witness(1), Witness(2), Witness(3)]),
        ..Circuit::default()
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 83, 219, 10, 128, 48, 8, 245, 210, 101, 159, 179,
        254, 160, 127, 137, 222, 138, 122, 236, 243, 27, 228, 64, 44, 232, 33, 7, 237, 128, 56,
        157, 147, 131, 103, 6, 0, 64, 184, 192, 201, 72, 206, 40, 177, 70, 174, 27, 197, 199, 111,
        24, 208, 175, 87, 44, 197, 145, 42, 224, 200, 5, 56, 230, 255, 240, 83, 189, 61, 117, 113,
        157, 31, 63, 236, 79, 147, 172, 77, 214, 73, 220, 139, 15, 106, 214, 168, 114, 249, 126,
        218, 214, 125, 153, 15, 54, 37, 90, 26, 155, 39, 227, 95, 223, 232, 230, 4, 247, 157, 215,
        56, 1, 153, 86, 63, 138, 44, 4, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}
