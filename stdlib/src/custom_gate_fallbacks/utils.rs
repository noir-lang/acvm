use core::num;

use acir::{
    circuit::{directives::Directive, Opcode},
    native_types::{Expression, Witness},
    FieldElement,
};

use crate::helpers::VariableStore;

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

pub(crate) fn radix_decomposition(
    gate: Expression,
    size: u32,
    radix: u32,
    mut num_witness: u32,
) -> (Vec<Opcode>, Vec<Witness>, u32) {
    let mut new_gates = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);

    // First create a witness for each bit
    let mut vector = Vec::with_capacity(size as usize);
    for _ in 0..size {
        vector.push(variables.new_variable())
    }

    // Next create a directive which computes those bits.
    new_gates.push(Opcode::Directive(Directive::ToLeRadix {
        a: gate.clone(),
        b: vector.clone(),
        radix: radix,
    }));

    // Now apply constraints to the bits such that they are the bit decomposition
    // of the input and each bit is actually a bit
    let mut binary_exprs = Vec::new();
    let mut decomp_constraint = gate;
    let mut two_pow: FieldElement = FieldElement::one();
    let two = FieldElement::from(2_i128);
    for &bit in &vector {
        // Bit constraint to ensure each bit is a zero or one; bit^2 - bit = 0
        let mut expr = Expression::default();
        expr.push_multiplication_term(FieldElement::one(), bit, bit);
        expr.push_addition_term(-FieldElement::one(), bit);
        binary_exprs.push(Opcode::Arithmetic(expr));

        // Constraint to ensure that the bits are constrained to be a bit decomposition
        // of the input
        // ie \sum 2^i * x_i = input
        decomp_constraint.push_addition_term(-two_pow, bit);
        two_pow = two * two_pow;
    }

    new_gates.extend(binary_exprs);
    decomp_constraint.sort(); // TODO: we have an issue open to check if this is needed. Ideally, we remove it.
    new_gates.push(Opcode::Arithmetic(decomp_constraint));

    (new_gates, vector, variables.finalize())
}

pub(crate) fn split_field_element_to_bytes(element: FieldElement, num_bytes: u32) {
    assert!(num_bytes <= 32);

    let mut value = element.to_be_bytes();
    value.resize(num_bytes as usize, 0);
}
