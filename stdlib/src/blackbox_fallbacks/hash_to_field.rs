use super::{
    blake2s::create_blake2s_constraint,
    utils::{byte_decomposition, round_to_nearest_byte},
    UInt32,
};
use acir::{
    circuit::Opcode,
    native_types::{Expression, Witness},
    FieldElement,
};

pub fn hash_to_field(
    inputs: Vec<(Expression, u32)>,
    outputs: Witness,
    mut num_witness: u32,
) -> (u32, Vec<Opcode>) {
    let mut new_gates = Vec::new();
    let mut new_inputs = Vec::new();

    // Decompose the input field elements into bytes and collect the resulting witnesses.
    for (witness, num_bits) in inputs {
        let num_bytes = round_to_nearest_byte(num_bits);
        let (extra_gates, inputs, updated_witness_counter) =
            byte_decomposition(witness, num_bytes, num_witness);
        new_gates.extend(extra_gates);
        new_inputs.extend(inputs);
        num_witness = updated_witness_counter;
    }

    let (result, num_witness, extra_gates) = create_blake2s_constraint(new_inputs, num_witness);
    new_gates.extend(extra_gates);

    let (result, extra_gates, num_witness) = field_from_be_bytes(&result, num_witness);
    new_gates.extend(extra_gates);

    // constrain the outputs to be the same as the result of the circuit
    let mut expr = Expression::from(outputs);
    expr.push_addition_term(-FieldElement::one(), result);
    new_gates.push(Opcode::Arithmetic(expr));
    (num_witness, new_gates)
}

fn field_from_be_bytes(result: &[Witness], num_witness: u32) -> (Witness, Vec<Opcode>, u32) {
    let mut new_gates = Vec::new();

    let (mut new_witness, extra_gates, num_witness) = UInt32::load_constant(0, num_witness);
    new_gates.extend(extra_gates);

    let (const_256, extra_gates, mut num_witness) = UInt32::load_constant(256, num_witness);
    new_gates.extend(extra_gates);

    for r in result.iter().take(result.len() - 1) {
        let (updated_witness, extra_gates, updated_witness_counter) =
            new_witness.add_with_overflow(&UInt32::new(*r), num_witness);
        new_gates.extend(extra_gates);
        let (updated_witness, extra_gates, updated_witness_counter) =
            updated_witness.mul_with_overflow(&const_256, updated_witness_counter);
        new_gates.extend(extra_gates);
        new_witness = updated_witness;
        num_witness = updated_witness_counter;
    }

    let (new_witness, extra_gates, num_witness) =
        new_witness.add_with_overflow(&UInt32::new(result[result.len() - 1]), num_witness);
    new_gates.extend(extra_gates);

    (new_witness.inner, new_gates, num_witness)
}
