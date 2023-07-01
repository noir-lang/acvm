use std::collections::BTreeMap;

use acir::{
    brillig_vm::{self, RegisterIndex},
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

use crate::{custom_gate_fallbacks::sha256_u32, helpers::VariableStore};

macro_rules! load_value {
    (
        $name:ident,
        $index:expr
    ) => {
        BrilligInputs::Single(Expression {
            // Input Register 0
            mul_terms: vec![],
            linear_combinations: vec![],
            q_c: FieldElement::from($name[$index]),
        })
    };
}

#[derive(Clone, Debug, Default)]
pub struct Sha256U32 {
    pub(crate) inner: Witness,
    width: u32,
}

impl Sha256U32 {
    pub fn new(witness: Witness) -> Self {
        Sha256U32 { inner: witness, width: 32 }
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

    pub fn ror(&self, target_rotation: u32, mut num_witness: u32) -> (Sha256U32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), self.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
                    // Input Register 1
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from(target_rotation as u128),
                }),
                BrilligInputs::Single(Expression {
                    // Input Register 2
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from(31_u128),
                }),
                BrilligInputs::Single(Expression {
                    // Input Register 3
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from(32_u128),
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![
                brillig_vm::Opcode::BinaryIntOp {
                    op: brillig_vm::BinaryIntOp::And,
                    bit_size: 32,
                    lhs: RegisterIndex::from(1),
                    rhs: RegisterIndex::from(2),
                    destination: RegisterIndex::from(2),
                },
                brillig_vm::Opcode::BinaryIntOp {
                    op: brillig_vm::BinaryIntOp::Sub,
                    bit_size: 32,
                    lhs: RegisterIndex::from(3),
                    rhs: RegisterIndex::from(2),
                    destination: RegisterIndex::from(3),
                },
                brillig_vm::Opcode::BinaryIntOp {
                    op: brillig_vm::BinaryIntOp::Shr,
                    bit_size: 32,
                    lhs: RegisterIndex::from(0),
                    rhs: RegisterIndex::from(2),
                    destination: RegisterIndex::from(2),
                },
                brillig_vm::Opcode::BinaryIntOp {
                    op: brillig_vm::BinaryIntOp::Shl,
                    bit_size: 32,
                    lhs: RegisterIndex::from(0),
                    rhs: RegisterIndex::from(3),
                    destination: RegisterIndex::from(3),
                },
                brillig_vm::Opcode::BinaryIntOp {
                    op: brillig_vm::BinaryIntOp::Or,
                    bit_size: 32,
                    lhs: RegisterIndex::from(2),
                    rhs: RegisterIndex::from(3),
                    destination: RegisterIndex::from(0),
                },
            ],
            predicate: None,
        });
        new_gates.push(brillig_opcode);

        // TODO: Deal with this contraint
        // let mut expr = Expression::from(new_witness);

        // expr.push_addition_term(
        //     -FieldElement::from(1_u128 << (self.width - target_rotation)),
        //     self.inner,
        // );

        // expr.push_addition_term(-FieldElement::from(1_u128 << (1 - self.width)), self.inner);

        // new_gates.push(Opcode::Arithmetic(expr));

        let num_witness = variables.finalize();

        (Sha256U32::new(new_witness), new_gates, num_witness)
    }

    pub(crate) fn rightshift(
        &self,
        bits: u32,
        mut num_witness: u32,
    ) -> (Sha256U32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), self.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
                    // Input Register 1
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from(bits as u128),
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![brillig_vm::Opcode::BinaryIntOp {
                op: brillig_vm::BinaryIntOp::Shr,
                bit_size: 32,
                lhs: RegisterIndex::from(0),
                rhs: RegisterIndex::from(1),
                destination: RegisterIndex::from(0),
            }],
            predicate: None,
        });
        new_gates.push(brillig_opcode);

        // TODO: Deal with this contraint
        // let mut expr = Expression::from(new_witness);

        let num_witness = variables.finalize();

        (Sha256U32::new(new_witness), new_gates, num_witness)
    }

    pub(crate) fn add(
        &self,
        rhs: Sha256U32,
        mut num_witness: u32,
    ) -> (Sha256U32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), self.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
                    // Input Register 1
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                    q_c: FieldElement::zero(),
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![brillig_vm::Opcode::BinaryIntOp {
                op: brillig_vm::BinaryIntOp::Add,
                bit_size: 32,
                lhs: RegisterIndex::from(0),
                rhs: RegisterIndex::from(1),
                destination: RegisterIndex::from(0),
            }],
            predicate: None,
        });
        new_gates.push(brillig_opcode);

        // let mut add_constraint = Expression::from(new_witness);

        // add_constraint.push_addition_term(-FieldElement::one(), self.inner);
        // add_constraint.push_addition_term(-FieldElement::one(), rhs.inner);
        // new_gates.push(Opcode::Arithmetic(add_constraint));

        let num_witness = variables.finalize();

        (Sha256U32::new(new_witness), new_gates, num_witness)
    }

    pub(crate) fn sub(
        &self,
        rhs: Sha256U32,
        mut num_witness: u32,
    ) -> (Sha256U32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), self.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
                    // Input Register 1
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                    q_c: FieldElement::zero(),
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![brillig_vm::Opcode::BinaryIntOp {
                op: brillig_vm::BinaryIntOp::Sub,
                bit_size: 32,
                lhs: RegisterIndex::from(0),
                rhs: RegisterIndex::from(1),
                destination: RegisterIndex::from(0),
            }],
            predicate: None,
        });
        new_gates.push(brillig_opcode);

        let mut sub_constraint = Expression::from(new_witness);

        sub_constraint.push_addition_term(-FieldElement::one(), self.inner);
        sub_constraint.push_addition_term(FieldElement::one(), rhs.inner);
        new_gates.push(Opcode::Arithmetic(sub_constraint));

        let num_witness = variables.finalize();

        (Sha256U32::new(new_witness), new_gates, num_witness)
    }

    pub(crate) fn and(
        &self,
        rhs: Sha256U32,
        mut num_witness: u32,
    ) -> (Sha256U32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), self.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
                    // Input Register 1
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                    q_c: FieldElement::zero(),
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![brillig_vm::Opcode::BinaryIntOp {
                op: brillig_vm::BinaryIntOp::And,
                bit_size: 32,
                lhs: RegisterIndex::from(0),
                rhs: RegisterIndex::from(1),
                destination: RegisterIndex::from(0),
            }],
            predicate: None,
        });
        new_gates.push(brillig_opcode);

        let and_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
            lhs: FunctionInput { witness: self.inner, num_bits: self.width },
            rhs: FunctionInput { witness: rhs.inner, num_bits: self.width },
            output: new_witness,
        });
        new_gates.push(and_opcode);

        let num_witness = variables.finalize();

        (Sha256U32::new(new_witness), new_gates, num_witness)
    }

    pub(crate) fn xor(
        &self,
        rhs: Sha256U32,
        mut num_witness: u32,
    ) -> (Sha256U32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), self.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
                    // Input Register 1
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                    q_c: FieldElement::zero(),
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![brillig_vm::Opcode::BinaryIntOp {
                op: brillig_vm::BinaryIntOp::Xor,
                bit_size: 32,
                lhs: RegisterIndex::from(0),
                rhs: RegisterIndex::from(1),
                destination: RegisterIndex::from(0),
            }],
            predicate: None,
        });
        new_gates.push(brillig_opcode);

        // let xor_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::XOR {
        //     lhs: FunctionInput { witness: self.inner, num_bits: self.width },
        //     rhs: FunctionInput { witness: rhs.inner, num_bits: self.width },
        //     output: new_witness,
        // });
        // new_gates.push(xor_opcode);

        let num_witness = variables.finalize();

        (Sha256U32::new(new_witness), new_gates, num_witness)
    }

    pub(crate) fn not(&self, mut num_witness: u32) -> (Sha256U32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    // Input Register 0
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), self.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
                    // Input Register 1
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::one(),
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![brillig_vm::Opcode::BinaryIntOp {
                op: brillig_vm::BinaryIntOp::Sub,
                bit_size: 32,
                lhs: RegisterIndex::from(1),
                rhs: RegisterIndex::from(0),
                destination: RegisterIndex::from(0),
            }],
            predicate: None,
        });
        new_gates.push(brillig_opcode);

        let mut not_constraint = Expression::from(new_witness);

        not_constraint.push_addition_term(FieldElement::one(), self.inner);
        not_constraint.q_c = -FieldElement::one();
        new_gates.push(Opcode::Arithmetic(not_constraint));

        let num_witness = variables.finalize();

        (Sha256U32::new(new_witness), new_gates, num_witness)
    }

    pub(crate) fn prepare_constants(mut num_witness: u32) -> (Vec<Sha256U32>, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);

        let state_1 = variables.new_variable();
        let state_2 = variables.new_variable();
        let state_3 = variables.new_variable();
        let state_4 = variables.new_variable();
        let state_5 = variables.new_variable();
        let state_6 = variables.new_variable();
        let state_7 = variables.new_variable();
        let state_8 = variables.new_variable();

        let init_constants: Vec<u128> = vec![
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                load_value!(init_constants, 0),
                load_value!(init_constants, 1),
                load_value!(init_constants, 2),
                load_value!(init_constants, 3),
                load_value!(init_constants, 4),
                load_value!(init_constants, 5),
                load_value!(init_constants, 6),
                load_value!(init_constants, 7),
            ],
            outputs: vec![
                BrilligOutputs::Simple(state_1),
                BrilligOutputs::Simple(state_2),
                BrilligOutputs::Simple(state_3),
                BrilligOutputs::Simple(state_4),
                BrilligOutputs::Simple(state_5),
                BrilligOutputs::Simple(state_6),
                BrilligOutputs::Simple(state_7),
                BrilligOutputs::Simple(state_8),
            ],
            foreign_call_results: vec![],
            bytecode: vec![brillig_vm::Opcode::Stop],
            predicate: None,
        });

        new_gates.push(brillig_opcode);

        let num_witness = variables.finalize();

        (
            vec![state_1, state_2, state_3, state_4, state_5, state_6, state_7, state_8]
                .into_iter()
                .map(Sha256U32::new)
                .collect(),
            new_gates,
            num_witness,
        )
    }

    pub(crate) fn prepare_round_constants(
        mut num_witness: u32,
    ) -> (Vec<Sha256U32>, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);

        let state_1 = variables.new_variable();
        let state_2 = variables.new_variable();
        let state_3 = variables.new_variable();
        let state_4 = variables.new_variable();
        let state_5 = variables.new_variable();
        let state_6 = variables.new_variable();
        let state_7 = variables.new_variable();
        let state_8 = variables.new_variable();
        let state_9 = variables.new_variable();
        let state_10 = variables.new_variable();
        let state_11 = variables.new_variable();
        let state_12 = variables.new_variable();
        let state_13 = variables.new_variable();
        let state_14 = variables.new_variable();
        let state_15 = variables.new_variable();
        let state_16 = variables.new_variable();
        let state_17 = variables.new_variable();
        let state_18 = variables.new_variable();
        let state_19 = variables.new_variable();
        let state_20 = variables.new_variable();
        let state_21 = variables.new_variable();
        let state_22 = variables.new_variable();
        let state_23 = variables.new_variable();
        let state_24 = variables.new_variable();
        let state_25 = variables.new_variable();
        let state_26 = variables.new_variable();
        let state_27 = variables.new_variable();
        let state_28 = variables.new_variable();
        let state_29 = variables.new_variable();
        let state_30 = variables.new_variable();
        let state_31 = variables.new_variable();
        let state_32 = variables.new_variable();
        let state_33 = variables.new_variable();
        let state_34 = variables.new_variable();
        let state_35 = variables.new_variable();
        let state_36 = variables.new_variable();
        let state_37 = variables.new_variable();
        let state_38 = variables.new_variable();
        let state_39 = variables.new_variable();
        let state_40 = variables.new_variable();
        let state_41 = variables.new_variable();
        let state_42 = variables.new_variable();
        let state_43 = variables.new_variable();
        let state_44 = variables.new_variable();
        let state_45 = variables.new_variable();
        let state_46 = variables.new_variable();
        let state_47 = variables.new_variable();
        let state_48 = variables.new_variable();
        let state_49 = variables.new_variable();
        let state_50 = variables.new_variable();
        let state_51 = variables.new_variable();
        let state_52 = variables.new_variable();
        let state_53 = variables.new_variable();
        let state_54 = variables.new_variable();
        let state_55 = variables.new_variable();
        let state_56 = variables.new_variable();
        let state_57 = variables.new_variable();
        let state_58 = variables.new_variable();
        let state_59 = variables.new_variable();
        let state_60 = variables.new_variable();
        let state_61 = variables.new_variable();
        let state_62 = variables.new_variable();
        let state_63 = variables.new_variable();
        let state_64 = variables.new_variable();

        let round_constants: Vec<u128> = vec![
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
            0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
            0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
            0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
            0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
            0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
            0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
            0xc67178f2,
        ];

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                load_value!(round_constants, 0),
                load_value!(round_constants, 1),
                load_value!(round_constants, 2),
                load_value!(round_constants, 3),
                load_value!(round_constants, 4),
                load_value!(round_constants, 5),
                load_value!(round_constants, 6),
                load_value!(round_constants, 7),
                load_value!(round_constants, 8),
                load_value!(round_constants, 9),
                load_value!(round_constants, 10),
                load_value!(round_constants, 11),
                load_value!(round_constants, 12),
                load_value!(round_constants, 13),
                load_value!(round_constants, 14),
                load_value!(round_constants, 15),
                load_value!(round_constants, 16),
                load_value!(round_constants, 17),
                load_value!(round_constants, 18),
                load_value!(round_constants, 19),
                load_value!(round_constants, 20),
                load_value!(round_constants, 21),
                load_value!(round_constants, 22),
                load_value!(round_constants, 23),
                load_value!(round_constants, 24),
                load_value!(round_constants, 25),
                load_value!(round_constants, 26),
                load_value!(round_constants, 27),
                load_value!(round_constants, 28),
                load_value!(round_constants, 29),
                load_value!(round_constants, 30),
                load_value!(round_constants, 31),
                load_value!(round_constants, 32),
                load_value!(round_constants, 33),
                load_value!(round_constants, 34),
                load_value!(round_constants, 35),
                load_value!(round_constants, 36),
                load_value!(round_constants, 37),
                load_value!(round_constants, 38),
                load_value!(round_constants, 39),
                load_value!(round_constants, 40),
                load_value!(round_constants, 41),
                load_value!(round_constants, 42),
                load_value!(round_constants, 43),
                load_value!(round_constants, 44),
                load_value!(round_constants, 45),
                load_value!(round_constants, 46),
                load_value!(round_constants, 47),
                load_value!(round_constants, 48),
                load_value!(round_constants, 49),
                load_value!(round_constants, 50),
                load_value!(round_constants, 51),
                load_value!(round_constants, 52),
                load_value!(round_constants, 53),
                load_value!(round_constants, 54),
                load_value!(round_constants, 55),
                load_value!(round_constants, 56),
                load_value!(round_constants, 57),
                load_value!(round_constants, 58),
                load_value!(round_constants, 59),
                load_value!(round_constants, 60),
                load_value!(round_constants, 61),
                load_value!(round_constants, 62),
                load_value!(round_constants, 63),
            ],
            outputs: vec![
                BrilligOutputs::Simple(state_1),
                BrilligOutputs::Simple(state_2),
                BrilligOutputs::Simple(state_3),
                BrilligOutputs::Simple(state_4),
                BrilligOutputs::Simple(state_5),
                BrilligOutputs::Simple(state_6),
                BrilligOutputs::Simple(state_7),
                BrilligOutputs::Simple(state_8),
                BrilligOutputs::Simple(state_9),
                BrilligOutputs::Simple(state_10),
                BrilligOutputs::Simple(state_11),
                BrilligOutputs::Simple(state_12),
                BrilligOutputs::Simple(state_13),
                BrilligOutputs::Simple(state_14),
                BrilligOutputs::Simple(state_15),
                BrilligOutputs::Simple(state_16),
                BrilligOutputs::Simple(state_17),
                BrilligOutputs::Simple(state_18),
                BrilligOutputs::Simple(state_19),
                BrilligOutputs::Simple(state_20),
                BrilligOutputs::Simple(state_21),
                BrilligOutputs::Simple(state_22),
                BrilligOutputs::Simple(state_23),
                BrilligOutputs::Simple(state_24),
                BrilligOutputs::Simple(state_25),
                BrilligOutputs::Simple(state_26),
                BrilligOutputs::Simple(state_27),
                BrilligOutputs::Simple(state_28),
                BrilligOutputs::Simple(state_29),
                BrilligOutputs::Simple(state_30),
                BrilligOutputs::Simple(state_31),
                BrilligOutputs::Simple(state_32),
                BrilligOutputs::Simple(state_33),
                BrilligOutputs::Simple(state_34),
                BrilligOutputs::Simple(state_35),
                BrilligOutputs::Simple(state_36),
                BrilligOutputs::Simple(state_37),
                BrilligOutputs::Simple(state_38),
                BrilligOutputs::Simple(state_39),
                BrilligOutputs::Simple(state_40),
                BrilligOutputs::Simple(state_41),
                BrilligOutputs::Simple(state_42),
                BrilligOutputs::Simple(state_43),
                BrilligOutputs::Simple(state_44),
                BrilligOutputs::Simple(state_45),
                BrilligOutputs::Simple(state_46),
                BrilligOutputs::Simple(state_47),
                BrilligOutputs::Simple(state_48),
                BrilligOutputs::Simple(state_49),
                BrilligOutputs::Simple(state_50),
                BrilligOutputs::Simple(state_51),
                BrilligOutputs::Simple(state_52),
                BrilligOutputs::Simple(state_53),
                BrilligOutputs::Simple(state_54),
                BrilligOutputs::Simple(state_55),
                BrilligOutputs::Simple(state_56),
                BrilligOutputs::Simple(state_57),
                BrilligOutputs::Simple(state_58),
                BrilligOutputs::Simple(state_59),
                BrilligOutputs::Simple(state_60),
                BrilligOutputs::Simple(state_61),
                BrilligOutputs::Simple(state_62),
                BrilligOutputs::Simple(state_63),
                BrilligOutputs::Simple(state_64),
            ],
            foreign_call_results: vec![],
            bytecode: vec![brillig_vm::Opcode::Stop],
            predicate: None,
        });

        new_gates.push(brillig_opcode);

        let num_witness = variables.finalize();

        (
            vec![
                state_1, state_2, state_3, state_4, state_5, state_6, state_7, state_8, state_9,
                state_10, state_11, state_12, state_13, state_14, state_15, state_16, state_17,
                state_18, state_19, state_20, state_21, state_22, state_23, state_24, state_25,
                state_26, state_27, state_28, state_29, state_30, state_31, state_32, state_33,
                state_34, state_35, state_36, state_37, state_38, state_39, state_40, state_41,
                state_42, state_43, state_44, state_45, state_46, state_47, state_48, state_49,
                state_50, state_51, state_52, state_53, state_54, state_55, state_56, state_57,
                state_58, state_59, state_60, state_61, state_62, state_63, state_64,
            ]
            .into_iter()
            .map(Sha256U32::new)
            .collect(),
            new_gates,
            num_witness,
        )
    }
}

// pub(crate) fn prepare_constants() -> Vec<FieldElement> {
//     vec![
//         FieldElement::from(1779033703_u128),
//         FieldElement::from(3144134277_u128),
//         FieldElement::from(1013904242_u128),
//         FieldElement::from(2773480762_u128),
//         FieldElement::from(1359893119_u128),
//         FieldElement::from(2600822924_u128),
//         FieldElement::from(528734635_u128),
//         FieldElement::from(1541459225_u128),
//     ]
// }

// struct StubbedPwg;

// impl PartialWitnessGenerator for StubbedPwg {
//     fn schnorr_verify(
//         &self,
//         _initial_witness: &mut WitnessMap,
//         _public_key_x: &FunctionInput,
//         _public_key_y: &FunctionInput,
//         _signature: &[FunctionInput],
//         _message: &[FunctionInput],
//         _output: &Witness,
//     ) -> Result<OpcodeResolution, OpcodeResolutionError> {
//         panic!("Path not trodden by this test")
//     }

//     fn pedersen(
//         &self,
//         _initial_witness: &mut WitnessMap,
//         _inputs: &[FunctionInput],
//         _domain_separator: u32,
//         _outputs: &[Witness],
//     ) -> Result<OpcodeResolution, OpcodeResolutionError> {
//         panic!("Path not trodden by this test")
//     }

//     fn fixed_base_scalar_mul(
//         &self,
//         _initial_witness: &mut WitnessMap,
//         _input: &FunctionInput,
//         _outputs: &[Witness],
//     ) -> Result<OpcodeResolution, OpcodeResolutionError> {
//         panic!("Path not trodden by this test")
//     }
// }
