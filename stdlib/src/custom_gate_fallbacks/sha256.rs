use acir::{
    brillig_vm::{self, BinaryFieldOp, RegisterIndex},
    circuit::{
        brillig::{Brillig, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, FunctionInput},
        Opcode,
    },
    native_types::{Expression, Witness},
    FieldElement,
};

use crate::helpers::VariableStore;

use super::utils::{byte_decomposition, round_to_nearest_byte};

pub fn sha256(
    inputs: Vec<(Expression, u32)>,
    outputs: Vec<Witness>,
    mut num_witness: u32,
) -> (u32, Vec<Opcode>) {
    let mut new_gates = Vec::new();
    let mut new_inputs = Vec::new();

    let mut total_num_bytes = 0;

    // Decompose the input field elements into bytes and collect the resulting witnesses.
    for (witness, num_bits) in inputs {
        let num_bytes = round_to_nearest_byte(num_bits);
        total_num_bytes += num_bytes;
        let (extra_gates, inputs, updated_witness_counter) =
            byte_decomposition(witness, num_bytes, num_witness);
        new_gates.extend(extra_gates);
        new_inputs.extend(inputs);
        num_witness = updated_witness_counter;
    }

    let (num_witness, extra_gates) =
        create_sha256_constraint(new_inputs, total_num_bytes, num_witness);
    new_gates.extend(extra_gates);
    (num_witness, new_gates)
}

fn create_sha256_constraint(
    mut input: Vec<Witness>,
    total_num_bytes: u32,
    num_witness: u32,
) -> (u32, Vec<Opcode>) {
    let mut new_gates = Vec::new();

    // pad the bytes according to sha256 padding rules.
    let message_bits = total_num_bytes * 8;
    let (mut num_witness, pad_witness, extra_gates) = pad(128, 8, num_witness);
    new_gates.extend(extra_gates);
    input.push(pad_witness);

    let bytes_per_block = 64;
    let num_bytes = total_num_bytes + 8;
    let num_blocks = num_bytes / bytes_per_block + ((num_bytes % bytes_per_block != 0) as u32);

    let num_total_bytes = num_blocks * bytes_per_block;
    for _ in num_bytes..num_total_bytes {
        let (updated_witness_counter, pad_witness, extra_gates) = pad(0, 8, num_witness);
        num_witness = updated_witness_counter;
        new_gates.extend(extra_gates);
        input.push(pad_witness);
    }

    let (num_witness, pad_witness, extra_gates) = pad(message_bits, 64, num_witness);
    new_gates.extend(extra_gates);

    let (extra_gates, pad_witness, num_witness) =
        byte_decomposition(pad_witness.into(), 8, num_witness);
    new_gates.extend(extra_gates);
    input.extend(pad_witness);

    (num_witness, new_gates)
}

fn pad(number: u32, bit_size: u32, mut num_witness: u32) -> (u32, Witness, Vec<Opcode>) {
    let mut new_gates = Vec::new();
    let mut variables = VariableStore::new(&mut num_witness);

    let pad = variables.new_variable();

    // TODO: Should this padding generated from brillig be constrained
    let brillig_opcode = Opcode::Brillig(Brillig {
        inputs: vec![
            BrilligInputs::Single(Expression {
                // Input Register 0
                mul_terms: vec![],
                linear_combinations: vec![],
                q_c: FieldElement::from(number as u128),
            }),
            BrilligInputs::Single(Expression::default()),
        ],
        outputs: vec![BrilligOutputs::Simple(pad)],
        foreign_call_results: vec![],
        bytecode: vec![brillig_vm::Opcode::BinaryFieldOp {
            op: BinaryFieldOp::Add,
            lhs: RegisterIndex::from(0),
            rhs: RegisterIndex::from(1),
            destination: RegisterIndex::from(0),
        }],
        predicate: None,
    });
    new_gates.push(brillig_opcode);

    let range = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
        input: FunctionInput { witness: pad, num_bits: bit_size },
    });
    new_gates.push(range);

    (num_witness, pad, new_gates)
}
