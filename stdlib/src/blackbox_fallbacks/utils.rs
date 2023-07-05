use acir::{
    circuit::{
        directives::Directive,
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Opcode,
    },
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
    let byte_shift: u32 = 256;
    for (i, v) in vector.iter().enumerate() {
        let range = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput { witness: *v, num_bits: 8 },
        });
        let scaling_factor_value = byte_shift.pow(num_bytes - 1 - i as u32);
        let scaling_factor = FieldElement::from(scaling_factor_value as u128);

        decomp_constraint.push_addition_term(-scaling_factor, *v);

        byte_exprs.push(range);
    }

    new_gates.extend(byte_exprs);
    decomp_constraint.sort();
    new_gates.push(Opcode::Arithmetic(decomp_constraint));

    (new_gates, vector, variables.finalize())
}
