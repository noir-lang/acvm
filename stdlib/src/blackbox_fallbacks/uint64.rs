use crate::impl_uint;

impl_uint!(UInt64, 64);
impl UInt64 {
    /// Load a [UInt64] from four [Witness]es each representing a [u8]
    pub(crate) fn from_witnesses(
        witnesses: &[Witness],
        mut num_witness: u32,
    ) -> (Vec<UInt64>, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let mut uint = Vec::new();

        for i in 0..witnesses.len() / 8 {
            let new_witness = variables.new_variable();
            let brillig_opcode = Opcode::Brillig(Brillig {
                inputs: vec![
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 8])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 8 + 1])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 8 + 2])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 8 + 3])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 8 + 4])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 8 + 5])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 8 + 6])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 8 + 7])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![],
                        q_c: FieldElement::from(8_u128),
                    }),
                ],
                outputs: vec![BrilligOutputs::Simple(new_witness)],
                foreign_call_results: vec![],
                bytecode: vec![
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Shl,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(8),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Add,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(1),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Shl,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(8),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Add,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(2),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Shl,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(8),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Add,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(3),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Shl,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(8),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Add,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(4),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Shl,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(8),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Add,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(5),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Shl,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(8),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Add,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(6),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Shl,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(8),
                        destination: RegisterIndex::from(0),
                    },
                    brillig::Opcode::BinaryIntOp {
                        op: brillig::BinaryIntOp::Add,
                        bit_size: 64,
                        lhs: RegisterIndex::from(0),
                        rhs: RegisterIndex::from(7),
                        destination: RegisterIndex::from(0),
                    },
                ],
                predicate: None,
            });
            uint.push(UInt64::new(new_witness));
            new_gates.push(brillig_opcode);
            let mut expr = Expression::from(new_witness);
            for j in 0..8 {
                let scaling_factor_value: u128 = 1 << (8 * (7 - j) as u32);
                let scaling_factor = FieldElement::from(scaling_factor_value);
                expr.push_addition_term(-scaling_factor, witnesses[i * 8 + j]);
            }

            new_gates.push(Opcode::Arithmetic(expr));
        }
        let num_witness = variables.finalize();

        (uint, new_gates, num_witness)
    }
}
