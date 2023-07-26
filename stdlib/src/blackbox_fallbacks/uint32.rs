use crate::impl_uint;

impl_uint!(UInt32, u32, 32);
impl UInt32 {
    /// Load a [UInt32] from four [Witness]es each representing a [u8]
    pub(crate) fn from_witnesses(
        witnesses: &[Witness],
        mut num_witness: u32,
    ) -> (Vec<UInt32>, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let mut uint = Vec::new();

        for i in 0..witnesses.len() / 4 {
            let new_witness = variables.new_variable();
            uint.push(UInt32::new(new_witness));
            let mut expr = Expression::from(new_witness);
            for j in 0..4 {
                let scaling_factor_value = 1 << (8 * (3 - j) as u32);
                let scaling_factor = FieldElement::from(scaling_factor_value as u128);
                expr.push_addition_term(-scaling_factor, witnesses[i * 4 + j]);
            }

            new_gates.push(Opcode::Arithmetic(expr));
        }
        let num_witness = variables.finalize();

        (uint, new_gates, num_witness)
    }

    /// Calculate and constrain `self` >= `rhs`
    //  This should be similar to its equivalent in the Noir repo
    pub(crate) fn more_than_eq_comparison(
        &self,
        rhs: &UInt32,
        mut num_witness: u32,
    ) -> (UInt32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();
        let q_witness = variables.new_variable();
        let r_witness = variables.new_variable();

        // calculate 2^32 + self - rhs
        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), self.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), rhs.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from(1_u128 << self.width),
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![
                BrilligOpcode::BinaryIntOp {
                    op: brillig::BinaryIntOp::Add,
                    bit_size: 127,
                    lhs: RegisterIndex::from(0),
                    rhs: RegisterIndex::from(2),
                    destination: RegisterIndex::from(0),
                },
                BrilligOpcode::BinaryIntOp {
                    op: brillig::BinaryIntOp::Sub,
                    bit_size: 127,
                    lhs: RegisterIndex::from(0),
                    rhs: RegisterIndex::from(1),
                    destination: RegisterIndex::from(0),
                },
            ],
            predicate: None,
        });
        new_gates.push(brillig_opcode);
        let num_witness = variables.finalize();

        // constrain subtraction
        let mut sub_constraint = Expression::from(self.inner);
        sub_constraint.push_addition_term(-FieldElement::one(), new_witness);
        sub_constraint.push_addition_term(-FieldElement::one(), rhs.inner);
        sub_constraint.q_c = FieldElement::from(1_u128 << self.width);
        new_gates.push(Opcode::Arithmetic(sub_constraint));

        let (two_pow_rhs, extra_gates, num_witness) =
            UInt32::load_constant(2_u128.pow(self.width), num_witness);
        new_gates.extend(extra_gates);

        // constraint 2^{max_bits} + a - b = q * 2^{max_bits} + r
        // q = 1 if a == b
        // q = 1 if a > b
        // q = 0 if a < b
        let quotient_opcode =
            Opcode::Directive(acir::circuit::directives::Directive::Quotient(QuotientDirective {
                a: new_witness.into(),
                b: two_pow_rhs.inner.into(),
                q: q_witness,
                r: r_witness,
                predicate: None,
            }));
        new_gates.push(quotient_opcode);

        // make sure r in 32 bit range and q is 1 bit
        let r_range_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: r_witness, num_bits: self.width },
        });
        let q_range_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: q_witness, num_bits: 1 },
        });
        new_gates.push(r_range_opcode);
        new_gates.push(q_range_opcode);

        (UInt32::new(q_witness), new_gates, num_witness)
    }

    /// Calculate and constrain `self` < `rhs`
    pub fn less_than_comparison(
        &self,
        rhs: &UInt32,
        num_witness: u32,
    ) -> (UInt32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let (mut comparison, extra_gates, num_witness) =
            self.more_than_eq_comparison(rhs, num_witness);
        new_gates.extend(extra_gates);
        comparison.width = 1;

        // `self` < `rhs` == not `self` >= `rhs`
        let (less_than, extra_gates, num_witness) = comparison.not(num_witness);
        new_gates.extend(extra_gates);

        (less_than, new_gates, num_witness)
    }
}
