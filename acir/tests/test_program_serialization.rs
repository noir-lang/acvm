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
        public_parameters: PublicInputs::default(),
        return_values: PublicInputs([Witness(3)].into()),
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 187, 13, 192, 32, 12, 68, 249, 100, 32, 27,
        219, 96, 119, 89, 37, 40, 176, 255, 8, 81, 36, 23, 72, 41, 195, 53, 215, 61, 221, 189, 35,
        132, 16, 195, 55, 217, 251, 244, 134, 127, 193, 184, 145, 149, 22, 22, 65, 101, 30, 173,
        12, 36, 188, 160, 88, 87, 1, 150, 94, 21, 21, 69, 229, 46, 74, 52, 148, 181, 89, 183, 6,
        134, 76, 3, 167, 24, 77, 135, 229, 125, 187, 32, 57, 231, 253, 154, 22, 151, 113, 113, 250,
        0, 123, 50, 20, 220, 112, 1, 0, 0,
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
        public_parameters: PublicInputs::default(),
        return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 138, 201, 9, 0, 0, 8, 195, 234, 241, 114, 255, 121,
        69, 69, 5, 49, 16, 242, 104, 21, 0, 161, 169, 218, 212, 83, 78, 229, 237, 11, 159, 214, 39,
        0, 55, 132, 28, 78, 72, 0, 0, 0,
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
        public_parameters: PublicInputs::default(),
        return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 138, 65, 10, 0, 64, 8, 2, 103, 183, 232, 255, 47,
        142, 138, 58, 68, 130, 168, 140, 10, 60, 90, 149, 118, 182, 79, 255, 105, 57, 140, 197,
        246, 39, 0, 246, 174, 71, 87, 84, 0, 0, 0,
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
        public_parameters: PublicInputs::default(),
        return_values: PublicInputs(BTreeSet::from([output])),
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 210, 233, 50, 66, 1, 24, 199, 225, 99, 223, 247,
        125, 15, 73, 146, 36, 73, 146, 36, 73, 194, 93, 184, 255, 75, 48, 122, 167, 167, 25, 103,
        230, 204, 83, 211, 151, 230, 253, 255, 126, 146, 36, 25, 73, 6, 79, 56, 193, 223, 254, 59,
        202, 166, 223, 199, 250, 239, 116, 255, 29, 231, 4, 39, 57, 197, 225, 59, 195, 89, 206,
        113, 158, 11, 92, 228, 18, 151, 185, 194, 85, 174, 113, 157, 27, 220, 228, 22, 183, 185,
        195, 93, 238, 113, 159, 7, 60, 228, 17, 83, 60, 230, 9, 79, 153, 230, 25, 51, 60, 103, 150,
        23, 204, 241, 146, 121, 94, 177, 192, 107, 22, 121, 195, 18, 111, 89, 230, 29, 43, 188,
        103, 149, 15, 172, 241, 145, 117, 62, 177, 193, 103, 54, 249, 194, 214, 191, 29, 227, 121,
        245, 189, 205, 55, 118, 248, 206, 46, 63, 216, 227, 39, 191, 248, 237, 115, 60, 209, 94,
        116, 23, 173, 69, 103, 209, 88, 244, 53, 108, 107, 198, 255, 136, 150, 162, 163, 104, 40,
        250, 137, 118, 162, 155, 104, 38, 122, 137, 86, 162, 147, 104, 36, 250, 136, 54, 162, 139,
        104, 34, 122, 136, 22, 162, 131, 104, 32, 246, 143, 237, 83, 201, 96, 243, 216, 59, 182,
        78, 219, 56, 99, 219, 172, 77, 115, 182, 204, 219, 176, 96, 187, 162, 205, 74, 182, 42,
        219, 168, 98, 155, 170, 77, 106, 182, 168, 219, 160, 225, 246, 77, 55, 111, 185, 113, 219,
        109, 59, 110, 218, 117, 203, 158, 27, 14, 111, 54, 188, 91, 226, 150, 127, 214, 93, 14,
        165, 212, 3, 0, 0,
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
        public_parameters: PublicInputs::default(),
        return_values: PublicInputs::default(),
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 148, 209, 10, 195, 32, 12, 69, 99, 109, 183, 126,
        78, 82, 181, 198, 183, 253, 202, 100, 22, 246, 178, 135, 49, 246, 253, 219, 152, 131, 176,
        250, 214, 244, 130, 68, 130, 28, 188, 55, 232, 8, 0, 3, 124, 101, 223, 171, 131, 181, 126,
        189, 83, 173, 184, 77, 100, 20, 89, 157, 30, 11, 27, 214, 213, 216, 86, 48, 15, 34, 239,
        143, 142, 141, 172, 229, 190, 23, 61, 83, 235, 40, 56, 215, 219, 179, 220, 31, 166, 113,
        74, 246, 154, 178, 186, 54, 119, 27, 173, 195, 217, 251, 18, 167, 66, 142, 206, 56, 165,
        204, 1, 125, 200, 51, 19, 83, 224, 112, 153, 216, 185, 194, 158, 99, 202, 41, 98, 34, 239,
        10, 45, 33, 185, 165, 194, 122, 189, 123, 161, 28, 203, 240, 23, 180, 150, 119, 201, 6,
        197, 28, 4, 114, 245, 172, 183, 178, 173, 162, 255, 97, 135, 121, 25, 104, 127, 111, 47,
        112, 131, 248, 45, 3, 5, 0, 0,
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
        public_parameters: PublicInputs::default(),
        return_values: PublicInputs::default(),
    };

    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();

    let expected_serialization: Vec<u8> = vec![
        31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 83, 219, 10, 128, 48, 8, 245, 210, 101, 159, 179,
        254, 160, 127, 137, 222, 138, 122, 236, 243, 91, 228, 64, 44, 232, 33, 7, 117, 64, 156,
        206, 201, 193, 51, 3, 0, 32, 156, 224, 100, 36, 103, 148, 88, 35, 215, 245, 226, 227, 59,
        116, 232, 215, 43, 150, 226, 72, 63, 224, 200, 5, 56, 230, 255, 240, 81, 189, 61, 117, 113,
        157, 31, 223, 236, 79, 149, 172, 78, 214, 72, 220, 138, 15, 106, 214, 168, 114, 249, 126,
        88, 230, 117, 26, 55, 54, 37, 90, 26, 155, 39, 227, 31, 223, 232, 230, 4, 215, 157, 63,
        176, 3, 89, 64, 134, 157, 36, 4, 0, 0,
    ];

    assert_eq!(bytes, expected_serialization)
}
