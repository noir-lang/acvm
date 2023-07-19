use crate::helpers::VariableStore;
use acir::{
    circuit::{
        directives::Directive,
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

fn round_to_nearest_mul_8(num_bits: u32) -> u32 {
    let remainder = num_bits % 8;

    if remainder == 0 {
        return num_bits;
    }

    num_bits + 8 - remainder
}

pub(crate) fn round_to_nearest_byte(num_bits: u32) -> u32 {
    round_to_nearest_mul_8(num_bits) / 8
}

pub(crate) fn boolean_expr(expr: &Expression, variables: &mut VariableStore) -> Expression {
    &mul_with_witness(expr, expr, variables) - expr
}

/// Returns an expression which represents a*b
/// If one has multiplicative term and the other is of degree one or more,
/// the function creates intermediate variables accordingly
pub(crate) fn mul_with_witness(
    a: &Expression,
    b: &Expression,
    variables: &mut VariableStore,
) -> Expression {
    let a_arith;
    let a_arith = if !a.mul_terms.is_empty() && !b.is_const() {
        let a_witness = variables.new_variable();
        a_arith = Expression::from(a_witness);
        &a_arith
    } else {
        a
    };
    let b_arith;
    let b_arith = if !b.mul_terms.is_empty() && !a.is_const() {
        if a == b {
            a_arith
        } else {
            let b_witness = variables.new_variable();
            b_arith = Expression::from(b_witness);
            &b_arith
        }
    } else {
        b
    };
    (a_arith * b_arith).expect("Both expressions are reduced to be degree<=1")
}

// Generates opcodes and directives to bit decompose the input `gate`
// Returns the bits and the updated witness counter
// TODO:Ideally, we return the updated witness counter, or we require the input
// TODO to be a VariableStore. We are not doing this because we want migration to
// TODO be less painful
pub(crate) fn bit_decomposition(
    gate: Expression,
    bit_size: u32,
    mut num_witness: u32,
) -> (Vec<Opcode>, Vec<Witness>, u32) {
    let mut new_gates = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);

    // First create a witness for each bit
    let mut bit_vector = Vec::with_capacity(bit_size as usize);
    for _ in 0..bit_size {
        bit_vector.push(variables.new_variable())
    }

    // Next create a directive which computes those bits.
    new_gates.push(Opcode::Directive(Directive::ToLeRadix {
        a: gate.clone(),
        b: bit_vector.clone(),
        radix: 2,
    }));

    // Now apply constraints to the bits such that they are the bit decomposition
    // of the input and each bit is actually a bit
    let mut binary_exprs = Vec::new();
    let mut bit_decomp_constraint = gate;
    let mut two_pow: FieldElement = FieldElement::one();
    let two = FieldElement::from(2_i128);
    for &bit in &bit_vector {
        // Bit constraint to ensure each bit is a zero or one; bit^2 - bit = 0
        let expr = boolean_expr(&bit.into(), &mut variables);
        binary_exprs.push(Opcode::Arithmetic(expr));

        // Constraint to ensure that the bits are constrained to be a bit decomposition
        // of the input
        // ie \sum 2^i * x_i = input
        bit_decomp_constraint.push_addition_term(-two_pow, bit);
        two_pow = two * two_pow;
    }

    new_gates.extend(binary_exprs);
    bit_decomp_constraint.sort(); // TODO: we have an issue open to check if this is needed. Ideally, we remove it.
    new_gates.push(Opcode::Arithmetic(bit_decomp_constraint));

    (new_gates, bit_vector, variables.finalize())
}

// TODO: Maybe this can be merged with `bit_decomposition`
pub(crate) fn byte_decomposition(
    gate: Expression,
    num_bytes: u32,
    mut num_witness: u32,
) -> (Vec<Opcode>, Vec<Witness>, u32) {
    let mut new_gates = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);

    // First create a witness for each byte
    let mut vector = Vec::with_capacity(num_bytes as usize);
    for _ in 0..num_bytes {
        vector.push(variables.new_variable())
    }

    // Next create a directive which computes those byte.
    new_gates.push(Opcode::Directive(Directive::ToLeRadix {
        a: gate.clone(),
        b: vector.clone(),
        radix: 256,
    }));
    vector.reverse();

    // Now apply constraints to the bytes such that they are the byte decomposition
    // of the input and each byte is actually a byte
    let mut byte_exprs = Vec::new();
    let mut decomp_constraint = gate;
    let byte_shift: u128 = 256;
    for (i, v) in vector.iter().enumerate() {
        let range = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: *v, num_bits: 8 },
        });
        let scaling_factor_value = byte_shift.pow(num_bytes - 1 - i as u32);
        let scaling_factor = FieldElement::from(scaling_factor_value);

        decomp_constraint.push_addition_term(-scaling_factor, *v);

        byte_exprs.push(range);
    }

    new_gates.extend(byte_exprs);
    decomp_constraint.sort();
    new_gates.push(Opcode::Arithmetic(decomp_constraint));

    (new_gates, vector, variables.finalize())
}
