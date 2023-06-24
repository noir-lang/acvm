use core::num;
use std::vec;

use acir::{
    circuit::{
        directives::Directive,
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

use crate::{fallback::range, helpers::VariableStore};

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

pub(crate) fn byte_decomposition(
    gate: Expression,
    num_bytes: u32,
    mut num_witness: u32,
) -> (Vec<Opcode>, Vec<Witness>, u32) {
    let mut new_gates = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);

    // First create a witness for each bit
    let mut vector = Vec::with_capacity(num_bytes as usize);
    for _ in 0..num_bytes {
        vector.push(variables.new_variable())
    }

    // Next create a directive which computes those bits.
    new_gates.push(Opcode::Directive(Directive::ToLeRadix {
        a: gate.clone(),
        b: vector.clone(),
        radix: 256,
    }));

    // Now apply constraints to the bytes such that they are the byte decomposition
    // of the input and each byte is actually a byte
    let mut byte_exprs = Vec::new();
    let mut decomp_constraint = gate;
    let byte_shift: u32 = 256;
    for i in 0..vector.len() {
        let range = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: vector[i], num_bits: 8 },
        });
        let scaling_factor_value = byte_shift.pow(num_bytes - 1 - i as u32);
        let scaling_factor = FieldElement::from(scaling_factor_value as u128);

        decomp_constraint.push_addition_term(scaling_factor, vector[i]);

        byte_exprs.push(range);
    }

    new_gates.extend(byte_exprs);
    decomp_constraint.sort(); // TODO: we have an issue open to check if this is needed. Ideally, we remove it.
    new_gates.push(Opcode::Arithmetic(decomp_constraint));

    (new_gates, vector, variables.finalize())
}

pub(crate) fn split_field_element_to_bytes(element: FieldElement, num_bytes: u32) {
    assert!(num_bytes <= 32);

    let mut value = element.to_be_bytes();
    value.resize(num_bytes as usize, 0);
}
