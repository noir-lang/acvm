use acir::{
    brillig_vm::{self, RegisterIndex},
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

use crate::helpers::VariableStore;

#[derive(Clone, Debug)]
pub(crate) struct Sha256U32 {
    inner: Witness,
}

impl Sha256U32 {
    fn new(witness: Witness) -> Self {
        Sha256U32 { inner: witness }
    }

    pub(crate) fn from_witnesses(
        witnesses: Vec<Witness>,
        mut num_witness: u32,
    ) -> (Vec<Sha256U32>, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let mut sha256u32 = Vec::new();

        for i in 0..witnesses.len() / 4 {
            let new_witness = variables.new_variable();
            let brillig_opcode = Opcode::Brillig(Brillig {
                inputs: vec![
                    BrilligInputs::Single(Expression {
                        // Input Register 0
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 4])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        // Input Register 1
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 4 + 1])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        // Input Register 2
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 4 + 2])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        // Input Register 3
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 4 + 3])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        // Input Register 4
                        mul_terms: vec![],
                        linear_combinations: vec![],
                        q_c: FieldElement::from(8_u128),
                    }),
                ],
                outputs: vec![BrilligOutputs::Simple(new_witness)],
                foreign_call_results: vec![],
                bytecode: vec![
                    brillig_vm::Opcode::BinaryIntOp {
                        op: brillig_vm::BinaryIntOp::Shl,
                        bit_size: 32,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(4),
                        destination: RegisterIndex::from(0),
                    },
                    brillig_vm::Opcode::BinaryIntOp {
                        op: brillig_vm::BinaryIntOp::Add,
                        bit_size: 32,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(1),
                        destination: RegisterIndex::from(0),
                    },
                    brillig_vm::Opcode::BinaryIntOp {
                        op: brillig_vm::BinaryIntOp::Shl,
                        bit_size: 32,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(4),
                        destination: RegisterIndex::from(0),
                    },
                    brillig_vm::Opcode::BinaryIntOp {
                        op: brillig_vm::BinaryIntOp::Add,
                        bit_size: 32,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(2),
                        destination: RegisterIndex::from(0),
                    },
                    brillig_vm::Opcode::BinaryIntOp {
                        op: brillig_vm::BinaryIntOp::Shl,
                        bit_size: 32,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(4),
                        destination: RegisterIndex::from(0),
                    },
                    brillig_vm::Opcode::BinaryIntOp {
                        op: brillig_vm::BinaryIntOp::Add,
                        bit_size: 32,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(3),
                        destination: RegisterIndex::from(0),
                    },
                ],
                predicate: None,
            });
            sha256u32.push(Sha256U32::new(new_witness));
            new_gates.push(brillig_opcode);

            // TODO: this seems to generating a lot of new witnesses
            let mut expr = Expression::from(new_witness);
            for j in 0..4 {
                let scaling_factor_value = 1 << (8 * (3 - j) as u32);
                let scaling_factor = FieldElement::from(scaling_factor_value as u128);
                expr.push_addition_term(-scaling_factor, witnesses[i * 4 + j]);
            }

            new_gates.push(Opcode::Arithmetic(expr));
        }

        let num_witness = variables.finalize();

        (sha256u32, new_gates, num_witness)
    }

    // pub(crate) fn prepare_constants(mut num_witness: u32) -> (Vec<Sha256U32>, u32) {
    //     let mut variables = VariableStore::new(&mut num_witness);

    //     let state_1 = variables.new_variable();
    //     let state_2 = variables.new_variable();
    //     let state_3 = variables.new_variable();
    //     let state_4 = variables.new_variable();
    //     let state_5 = variables.new_variable();
    //     let state_6 = variables.new_variable();
    //     let state_7 = variables.new_variable();
    //     let state_8 = variables.new_variable();

    //     let brillig_opcode = Opcode::Brillig(Brillig {
    //         inputs: vec![
    //             BrilligInputs::Single(Expression {
    //                 // Input Register 0
    //                 mul_terms: vec![],
    //                 linear_combinations: vec![],
    //                 q_c: FieldElement::from(1779033703_u128),
    //             }),
    //             BrilligInputs::Single(Expression {
    //                 // Input Register 1
    //                 mul_terms: vec![],
    //                 linear_combinations: vec![],
    //                 q_c: FieldElement::from(3144134277_u128),
    //             }),
    //             BrilligInputs::Single(Expression {
    //                 // Input Register 2
    //                 mul_terms: vec![],
    //                 linear_combinations: vec![],
    //                 q_c: FieldElement::from(1013904242_u128),
    //             }),
    //             BrilligInputs::Single(Expression {
    //                 // Input Register 3
    //                 mul_terms: vec![],
    //                 linear_combinations: vec![],
    //                 q_c: FieldElement::from(2773480762_u128),
    //             }),
    //             BrilligInputs::Single(Expression {
    //                 // Input Register 4
    //                 mul_terms: vec![],
    //                 linear_combinations: vec![],
    //                 q_c: FieldElement::from(1359893119_u128),
    //             }),
    //             BrilligInputs::Single(Expression {
    //                 // Input Register 5
    //                 mul_terms: vec![],
    //                 linear_combinations: vec![],
    //                 q_c: FieldElement::from(2600822924_u128),
    //             }),
    //             BrilligInputs::Single(Expression {
    //                 // Input Register 6
    //                 mul_terms: vec![],
    //                 linear_combinations: vec![],
    //                 q_c: FieldElement::from(528734635_u128),
    //             }),
    //             BrilligInputs::Single(Expression {
    //                 // Input Register 7
    //                 mul_terms: vec![],
    //                 linear_combinations: vec![],
    //                 q_c: FieldElement::from(1541459225_u128),
    //             }),
    //         ],
    //         outputs: vec![
    //             BrilligOutputs::Simple(state_1),
    //             BrilligOutputs::Simple(state_2),
    //             BrilligOutputs::Simple(state_3),
    //             BrilligOutputs::Simple(state_4),
    //             BrilligOutputs::Simple(state_5),
    //             BrilligOutputs::Simple(state_6),
    //             BrilligOutputs::Simple(state_7),
    //             BrilligOutputs::Simple(state_8),
    //         ],
    //         foreign_call_results: vec![],
    //         bytecode: vec![],
    //         predicate: None,
    //     });

    //     let num_witness = variables.finalize();

    //     (
    //         vec![state_1, state_2, state_3, state_4, state_5, state_6, state_7, state_8]
    //             .into_iter()
    //             .map(Sha256U32::new)
    //             .collect(),
    //         num_witness,
    //     )
    // }
}

pub(crate) fn prepare_constants() -> Vec<FieldElement> {
    vec![
        FieldElement::from(1779033703_u128),
        FieldElement::from(3144134277_u128),
        FieldElement::from(1013904242_u128),
        FieldElement::from(2773480762_u128),
        FieldElement::from(1359893119_u128),
        FieldElement::from(2600822924_u128),
        FieldElement::from(528734635_u128),
        FieldElement::from(1541459225_u128),
    ]
}
