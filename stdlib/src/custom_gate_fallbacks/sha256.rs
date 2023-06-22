use acir::{
    circuit::{opcodes::FunctionInput, Opcode},
    native_types::{Expression, Witness},
    FieldElement,
};

use crate::{fallback::range, helpers::VariableStore};

use super::utils::{radix_decomposition, round_to_nearest_byte};

pub fn sha256(
    inputs: Vec<(Expression, u32)>,
    outputs: Vec<Witness>,
    mut num_witness: u32,
) -> (u32, Vec<Opcode>) {
    let mut new_gates = Vec::new();

    let mut total_num_bytes = 0;

    for (witness, num_bits) in &inputs {
        let num_bytes = round_to_nearest_byte(*num_bits);
        total_num_bytes += num_bytes;
        let (extra_gates, _, updated_witness_counter) =
            radix_decomposition(witness.clone(), num_bytes, 256, num_witness);
        new_gates.extend(extra_gates);
        num_witness = updated_witness_counter;
    }

    let output_bytes = create_sha256_constraint(inputs, total_num_bytes, num_witness);
    (0, Vec::new())
}

fn create_sha256_constraint(
    input: Vec<(Expression, u32)>,
    total_num_bytes: u32,
    mut num_witness: u32,
) {
    let mut new_gates = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);

    let message_bits = total_num_bytes * 8;
    let (num_witness, gates) = pad(128, 8, num_witness);
    new_gates.extend(gates);

    let bytes_per_block = 64;
    let num_bytes = total_num_bytes + 8;
    let num_blocks = num_bytes / bytes_per_block + ((num_bytes % bytes_per_block != 0) as u32);

    let num_total_bytes = num_blocks * bytes_per_block;
    for _ in num_bytes..num_total_bytes {
        let (num_witness, gates) = pad(0, 8, num_witness);
        new_gates.extend(gates);
    }

    let (num_witness, gates) = pad(message_bits, 64, num_witness);
}

fn pad(number: u32, size: u32, mut num_witness: u32) -> (u32, Vec<Opcode>) {
    let mut new_gates = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);

    let pad = variables.new_variable();
    let mut pad_expr = Expression::default();
    pad_expr.push_addition_term(FieldElement::from(number as u128), pad);
    new_gates.push(Opcode::Arithmetic(pad_expr.clone()));
    let num_witness = variables.finalize();
    let (num_witness, gates) = range(pad_expr, size, num_witness);
    new_gates.extend(gates);

    (num_witness, new_gates)
}
