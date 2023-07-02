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

use crate::helpers::VariableStore;

#[derive(Clone, Debug)]
pub struct Sha256U32 {
    pub(crate) inner: Witness,
    width: u32,
}

impl Default for Sha256U32 {
    fn default() -> Self {
        Sha256U32 { inner: Witness(1), width: 32 }
    }
}

impl Sha256U32 {
    pub fn new(witness: Witness) -> Self {
        Sha256U32 { inner: witness, width: 32 }
    }

    fn load_constant(constant: u128, mut num_witness: u32) -> (Sha256U32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![BrilligInputs::Single(Expression {
                // Input Register 0
                mul_terms: vec![],
                linear_combinations: vec![],
                q_c: FieldElement::from(constant),
            })],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![brillig_vm::Opcode::Stop],
            predicate: None,
        });

        new_gates.push(brillig_opcode);

        let num_witness = variables.finalize();

        (Sha256U32::new(new_witness), new_gates, num_witness)
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

        // TODO: see why this doesn't work
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

        // TODO: see why this doesn't work
        // let mut sub_constraint = Expression::from(new_witness);

        // sub_constraint.push_addition_term(-FieldElement::one(), self.inner);
        // sub_constraint.push_addition_term(FieldElement::one(), rhs.inner);
        // new_gates.push(Opcode::Arithmetic(sub_constraint));

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

        let xor_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::XOR {
            lhs: FunctionInput { witness: self.inner, num_bits: self.width },
            rhs: FunctionInput { witness: rhs.inner, num_bits: self.width },
            output: new_witness,
        });
        new_gates.push(xor_opcode);

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
                    q_c: FieldElement::from((1_u128 << self.width) - 1),
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

        // TODO: add constraint
        // let mut not_constraint = Expression::from(new_witness);

        // not_constraint.push_addition_term(FieldElement::one(), self.inner);
        // not_constraint.q_c = -FieldElement::one();
        // new_gates.push(Opcode::Arithmetic(not_constraint));

        let num_witness = variables.finalize();

        (Sha256U32::new(new_witness), new_gates, num_witness)
    }

    pub(crate) fn prepare_constants(mut num_witness: u32) -> (Vec<Sha256U32>, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut new_witnesses = Vec::new();

        let init_constants: Vec<u128> = vec![
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];

        for i in init_constants {
            let (new_witness, extra_gates, updated_witness_counter) =
                Sha256U32::load_constant(i, num_witness);
            new_gates.extend(extra_gates);
            new_witnesses.push(new_witness);
            num_witness = updated_witness_counter;
        }

        (new_witnesses, new_gates, num_witness)
    }

    pub(crate) fn prepare_round_constants(
        mut num_witness: u32,
    ) -> (Vec<Sha256U32>, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut new_witnesses = Vec::new();

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

        for i in round_constants {
            let (new_witness, extra_gates, updated_witness_counter) =
                Sha256U32::load_constant(i, num_witness);
            new_gates.extend(extra_gates);
            new_witnesses.push(new_witness);
            num_witness = updated_witness_counter;
        }

        (new_witnesses, new_gates, num_witness)
    }
}
