use crate::helpers::VariableStore;
use acir::{
    brillig_vm::{self, RegisterIndex},
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        directives::QuotientDirective,
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

/// WU32 (Witness u32) is a field element that represents a u32 integer
/// It has a inner field of type [Witness] that points to the field element and width = 32
// TODO: This can be generalized to u8, u64 and others if needed.
#[derive(Clone, Debug)]
pub struct WU32 {
    pub inner: Witness,
    width: u32,
}

/// The default has [Witness(1)]
impl Default for WU32 {
    fn default() -> Self {
        WU32 { inner: Witness(1), width: 32 }
    }
}

impl WU32 {
    /// Initialize A new [WU32] type with a [Witness]
    pub fn new(witness: Witness) -> Self {
        WU32 { inner: witness, width: 32 }
    }

    /// Load a [u128] constant into the circuit
    // TODO: This is currently a u128 instead of a u32 cus
    // in some cases we want to load 2^32 which does not fit in u32
    pub(crate) fn load_constant(constant: u128, mut num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![BrilligInputs::Single(Expression {
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

        (WU32::new(new_witness), new_gates, num_witness)
    }

    /// load a [WU32] from four [Witness] each representing a [u8]
    pub(crate) fn from_witnesses(
        witnesses: &[Witness],
        mut num_witness: u32,
    ) -> (Vec<WU32>, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let mut wu32 = Vec::new();

        for i in 0..witnesses.len() / 4 {
            let new_witness = variables.new_variable();
            let brillig_opcode = Opcode::Brillig(Brillig {
                inputs: vec![
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 4])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 4 + 1])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 4 + 2])],
                        q_c: FieldElement::zero(),
                    }),
                    BrilligInputs::Single(Expression {
                        mul_terms: vec![],
                        linear_combinations: vec![(FieldElement::one(), witnesses[i * 4 + 3])],
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
            wu32.push(WU32::new(new_witness));
            new_gates.push(brillig_opcode);
            let mut expr = Expression::from(new_witness);
            for j in 0..4 {
                let scaling_factor_value = 1 << (8 * (3 - j) as u32);
                let scaling_factor = FieldElement::from(scaling_factor_value as u128);
                expr.push_addition_term(-scaling_factor, witnesses[i * 4 + j]);
            }

            new_gates.push(Opcode::Arithmetic(expr));
        }
        let num_witness = variables.finalize();

        (wu32, new_gates, num_witness)
    }

    /// Returns the quotient and remainder such that lhs = rhs * quotient + remainder
    // This should be the same as its equivalent in the Noir repo
    // TODO: In Noir there's a constrain that "Constrain r < rhs"
    // Is it necessary here
    pub fn euclidean_division(
        lhs: &WU32,
        rhs: &WU32,
        mut num_witness: u32,
    ) -> (WU32, WU32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let q_witness = variables.new_variable();
        let r_witness = variables.new_variable();

        // compute quotient using directive function
        let quotient_opcode =
            Opcode::Directive(acir::circuit::directives::Directive::Quotient(QuotientDirective {
                a: lhs.inner.into(),
                b: rhs.inner.into(),
                q: q_witness,
                r: r_witness,
                predicate: None,
            }));
        new_gates.push(quotient_opcode);

        // make sure r and q are in 32 bit range
        let r_range_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: r_witness, num_bits: lhs.width },
        });
        let q_range_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: q_witness, num_bits: lhs.width },
        });
        new_gates.push(r_range_opcode);
        new_gates.push(q_range_opcode);

        // constrain lhs = rhs * quotient + remainder
        let rhs_expr = Expression::from(rhs.inner);
        let lhs_constraint = Expression::from(lhs.inner);
        let rhs_constraint = &rhs_expr * &Expression::from(q_witness);
        let rhs_constraint = &rhs_constraint.unwrap() + &Expression::from(r_witness);
        let div_euclidean = &lhs_constraint - &rhs_constraint;
        new_gates.push(Opcode::Arithmetic(div_euclidean));
        let num_witness = variables.finalize();

        (WU32::new(q_witness), WU32::new(r_witness), new_gates, num_witness)
    }

    /// rotate right `rotation` bits. `(x >> rotation) | (x << (width - rotation))`
    // Switched `or` with `add` here
    // This should be the same as `u32.rotate_right(rotation)` in rust stdlib
    pub fn ror(&self, rotation: u32, num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();

        let (left_shift, extra_gates, num_witness) = self.leftshift(32 - rotation, num_witness);
        new_gates.extend(extra_gates);
        let (right_shift, extra_gates, num_witness) = self.rightshift(rotation, num_witness);
        new_gates.extend(extra_gates);
        let (result, extra_gates, num_witness) = left_shift.add(&right_shift, num_witness);
        new_gates.extend(extra_gates);

        (result, new_gates, num_witness)
    }

    /// left shift by `bits`
    pub fn leftshift(&self, bits: u32, num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let (two_pow_rhs, extra_gates, num_witness) =
            WU32::load_constant(2_u128.pow(bits), num_witness);
        new_gates.extend(extra_gates);
        let (left_shift, extra_gates, num_witness) = self.mul(&two_pow_rhs, num_witness);
        new_gates.extend(extra_gates);

        (left_shift, new_gates, num_witness)
    }

    /// right shift by `bits`
    pub fn rightshift(&self, bits: u32, num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let (two_pow_rhs, extra_gates, num_witness) =
            WU32::load_constant(2_u128.pow(bits), num_witness);
        new_gates.extend(extra_gates);
        let (right_shift, _, extra_gates, num_witness) =
            WU32::euclidean_division(self, &two_pow_rhs, num_witness);
        new_gates.extend(extra_gates);

        (right_shift, new_gates, num_witness)
    }

    /// Caculate and constrain `self` + `rhs`
    pub fn add(&self, rhs: &WU32, mut num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        // calculate `self` + `rhs` with overflow
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
                bit_size: 127,
                lhs: RegisterIndex::from(0),
                rhs: RegisterIndex::from(1),
                destination: RegisterIndex::from(0),
            }],
            predicate: None,
        });
        new_gates.push(brillig_opcode);
        let num_witness = variables.finalize();

        // constrain addition
        let mut add_expr = Expression::from(new_witness);
        add_expr.push_addition_term(-FieldElement::one(), self.inner);
        add_expr.push_addition_term(-FieldElement::one(), rhs.inner);
        new_gates.push(Opcode::Arithmetic(add_expr));

        // mod 2^width to get final result as the remainder
        let (two_pow_width, extra_gates, num_witness) =
            WU32::load_constant(2_u128.pow(self.width), num_witness);
        new_gates.extend(extra_gates);
        let (_, add_mod, extra_gates, num_witness) =
            WU32::euclidean_division(&WU32::new(new_witness), &two_pow_width, num_witness);
        new_gates.extend(extra_gates);

        (add_mod, new_gates, num_witness)
    }

    /// Caculate and constrain `self` - `rhs`
    pub fn sub(&self, rhs: &WU32, mut num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        // calculate 2^32 + self - rhs to avoid overflow
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
                BrilligInputs::Single(Expression {
                    // Input Register 2
                    mul_terms: vec![],
                    linear_combinations: vec![],
                    q_c: FieldElement::from(1_u128 << self.width),
                }),
            ],
            outputs: vec![BrilligOutputs::Simple(new_witness)],
            foreign_call_results: vec![],
            bytecode: vec![
                brillig_vm::Opcode::BinaryIntOp {
                    op: brillig_vm::BinaryIntOp::Add,
                    bit_size: 127,
                    lhs: RegisterIndex::from(0),
                    rhs: RegisterIndex::from(2),
                    destination: RegisterIndex::from(0),
                },
                brillig_vm::Opcode::BinaryIntOp {
                    op: brillig_vm::BinaryIntOp::Sub,
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

        // mod 2^width to get final result as the remainder
        let (two_pow_width, extra_gates, num_witness) =
            WU32::load_constant(2_u128.pow(self.width), num_witness);
        new_gates.extend(extra_gates);
        let (_, sub_mod, extra_gates, num_witness) =
            WU32::euclidean_division(&WU32::new(new_witness), &two_pow_width, num_witness);
        new_gates.extend(extra_gates);

        (sub_mod, new_gates, num_witness)
    }

    /// Calculate and constrain `self` * `rhs`
    pub(crate) fn mul(&self, rhs: &WU32, mut num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        // calulate `self` * `rhs` with overflow
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
                op: brillig_vm::BinaryIntOp::Mul,
                bit_size: 127,
                lhs: RegisterIndex::from(0),
                rhs: RegisterIndex::from(1),
                destination: RegisterIndex::from(0),
            }],
            predicate: None,
        });
        new_gates.push(brillig_opcode);
        let num_witness = variables.finalize();

        // constrain mul
        let mut mul_constraint = Expression::from(new_witness);
        mul_constraint.push_multiplication_term(-FieldElement::one(), self.inner, rhs.inner);
        new_gates.push(Opcode::Arithmetic(mul_constraint));

        // mod 2^width to get final result as the remainder
        let (two_pow_rhs, extra_gates, num_witness) =
            WU32::load_constant(2_u128.pow(self.width), num_witness);
        new_gates.extend(extra_gates);
        let (_, mul_mod, extra_gates, num_witness) =
            WU32::euclidean_division(&WU32::new(new_witness), &two_pow_rhs, num_witness);
        new_gates.extend(extra_gates);

        (mul_mod, new_gates, num_witness)
    }

    /// Calculate and constrain `self` and `rhs`
    pub(crate) fn and(&self, rhs: &WU32, mut num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
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
        let num_witness = variables.finalize();

        let and_opcode = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
            lhs: FunctionInput { witness: self.inner, num_bits: self.width },
            rhs: FunctionInput { witness: rhs.inner, num_bits: self.width },
            output: new_witness,
        });
        new_gates.push(and_opcode);

        (WU32::new(new_witness), new_gates, num_witness)
    }

    /// Calculate and constrain `self` xor `rhs`
    pub(crate) fn xor(&self, rhs: WU32, mut num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
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

        (WU32::new(new_witness), new_gates, num_witness)
    }

    /// Calculate and constrain not `self`
    pub(crate) fn not(&self, mut num_witness: u32) -> (WU32, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut variables = VariableStore::new(&mut num_witness);
        let new_witness = variables.new_variable();

        let brillig_opcode = Opcode::Brillig(Brillig {
            inputs: vec![
                BrilligInputs::Single(Expression {
                    mul_terms: vec![],
                    linear_combinations: vec![(FieldElement::one(), self.inner)],
                    q_c: FieldElement::zero(),
                }),
                BrilligInputs::Single(Expression {
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
        let num_witness = variables.finalize();

        let mut not_constraint = Expression::from(new_witness);
        not_constraint.push_addition_term(FieldElement::one(), self.inner);
        not_constraint.q_c = -FieldElement::from((1_u128 << self.width) - 1);
        new_gates.push(Opcode::Arithmetic(not_constraint));

        (WU32::new(new_witness), new_gates, num_witness)
    }
}
