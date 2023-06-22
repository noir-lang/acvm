use acir::{
    circuit::{opcodes::FunctionInput, Opcode},
    native_types::{Expression, Witness},
    FieldElement,
};

use crate::helpers::VariableStore;

use super::utils::{radix_decomposition, round_to_nearest_byte};

pub fn sha256(
    inputs: Vec<(Expression, u32)>,
    outputs: Vec<Witness>,
    mut num_witness: u32,
) -> (u32, Vec<Opcode>) {
    let mut new_gates = Vec::new();

    let mut calculate_total_bytes_exprs = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);
    let total_bytes_witness = variables.new_variable();
    let mut total_bytes = Expression::default();
    for (_, num_bits) in &inputs {
        let num_bytes = round_to_nearest_byte(*num_bits);
        total_bytes.push_addition_term(FieldElement::from(num_bytes as u128), total_bytes_witness);
    }
    calculate_total_bytes_exprs.push(Opcode::Arithmetic(total_bytes));
    let mut num_witness = variables.finalize();

    new_gates.extend(calculate_total_bytes_exprs);

    for (witness, num_bits) in &inputs {
        let num_bytes = round_to_nearest_byte(*num_bits);
        let (extra_gates, _, updated_witness_counter) =
            radix_decomposition(witness.clone(), num_bytes, 256, num_witness);
        new_gates.extend(extra_gates);
        num_witness = updated_witness_counter;
    }

    let output_bytes = create_sha256_constraint(inputs, total_bytes_witness, num_witness);
    (0, Vec::new())
}

fn create_sha256_constraint(
    input: Vec<(Expression, u32)>,
    total_bytes_witness: Witness,
    mut num_witness: u32,
) {
    let mut variables = VariableStore::new(&mut num_witness);
}
