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

pub use super::sha256_u32::Sha256U32;
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

    let (result, num_witness, extra_gates) =
        create_sha256_constraint(new_inputs, total_num_bytes, num_witness);
    new_gates.extend(extra_gates);

    for i in 0..outputs.len() {
        let mut expr = Expression::from(outputs[i]);
        expr.push_addition_term(-FieldElement::one(), result[i]);
        new_gates.push(Opcode::Arithmetic(expr));
    }
    (num_witness, new_gates)
}

fn create_sha256_constraint(
    mut input: Vec<Witness>,
    total_num_bytes: u32,
    num_witness: u32,
) -> (Vec<Witness>, u32, Vec<Opcode>) {
    let mut new_gates = Vec::new();

    // pad the bytes according to sha256 padding rules.
    let message_bits = total_num_bytes * 8;
    let (mut num_witness, pad_witness, extra_gates) = pad(128, 8, num_witness);
    new_gates.extend(extra_gates);
    input.push(pad_witness);

    let bytes_per_block = 64;
    let num_bytes = (input.len() + 8) as u32;
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

    // turn witness into u32 and load sha256 state
    let (input, extra_gates, num_witness) = Sha256U32::from_witnesses(input, num_witness);
    new_gates.extend(extra_gates);

    let (mut rolling_hash, extra_gates, num_witness) = Sha256U32::prepare_constants(num_witness);
    new_gates.extend(extra_gates);

    let (round_constants, extra_gates, mut num_witness) =
        Sha256U32::prepare_round_constants(num_witness);
    new_gates.extend(extra_gates);

    let input: Vec<Vec<Sha256U32>> = input.chunks(16).map(|block| block.to_vec()).collect();

    // process sha256 blocks
    for i in &input {
        let (new_rolling_hash, extra_gates, updated_witness_counter) =
            sha256_block(i, rolling_hash.clone(), round_constants.clone(), num_witness);
        new_gates.extend(extra_gates);
        num_witness = updated_witness_counter;
        rolling_hash = new_rolling_hash;
    }

    let (extra_gates, byte1, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[0].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, byte2, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[1].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, byte3, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[2].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, byte4, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[3].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, byte5, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[4].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, byte6, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[5].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, byte7, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[6].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, byte8, num_witness) =
        byte_decomposition(Expression::from(rolling_hash[7].inner), 4, num_witness);
    new_gates.extend(extra_gates);

    let result = vec![byte1, byte2, byte3, byte4, byte5, byte6, byte7, byte8]
        .into_iter()
        .flatten()
        .collect();

    println!("{:?}", result);

    (result, num_witness, new_gates)
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

fn sha256_block(
    input: &[Sha256U32],
    rolling_hash: Vec<Sha256U32>,
    round_constants: Vec<Sha256U32>,
    mut num_witness: u32,
) -> (Vec<Sha256U32>, Vec<Opcode>, u32) {
    let mut new_gates = Vec::new();
    let mut w = Vec::new();
    w.extend(input.to_owned());
    w.resize(64, Sha256U32::default());

    for i in 16..64 {
        // calculate s0
        let (a1, extra_gates, updated_witness_counter) = w[i - 15].ror(7, num_witness);
        new_gates.extend(extra_gates);
        let (a2, extra_gates, updated_witness_counter) = w[i - 15].ror(18, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (a3, extra_gates, updated_witness_counter) =
            w[i - 15].rightshift(3, updated_witness_counter);
        new_gates.extend(extra_gates);

        let (a4, extra_gates, updated_witness_counter) = a1.xor(a2, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (s0, extra_gates, updated_witness_counter) = a4.xor(a3, updated_witness_counter);
        new_gates.extend(extra_gates);

        // calculate s1
        let (b1, extra_gates, updated_witness_counter) = w[i - 2].ror(17, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (b2, extra_gates, updated_witness_counter) = w[i - 2].ror(19, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (b3, extra_gates, updated_witness_counter) =
            w[i - 2].rightshift(10, updated_witness_counter);
        new_gates.extend(extra_gates);

        let (b4, extra_gates, updated_witness_counter) = b1.xor(b2, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (s1, extra_gates, updated_witness_counter) = b4.xor(b3, updated_witness_counter);
        new_gates.extend(extra_gates);

        // calculate w[i]

        let (c1, extra_gates, updated_witness_counter) =
            w[i - 16].add(w[i - 7].clone(), updated_witness_counter);
        new_gates.extend(extra_gates);
        let (c2, extra_gates, updated_witness_counter) = c1.add(s0, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (c3, extra_gates, updated_witness_counter) = c2.add(s1, updated_witness_counter);
        new_gates.extend(extra_gates);

        w[i] = c3;
        num_witness = updated_witness_counter;
    }

    let mut a = rolling_hash[0].clone();
    let mut b = rolling_hash[1].clone();
    let mut c = rolling_hash[2].clone();
    let mut d = rolling_hash[3].clone();
    let mut e = rolling_hash[4].clone();
    let mut f = rolling_hash[5].clone();
    let mut g = rolling_hash[6].clone();
    let mut h = rolling_hash[7].clone();

    #[allow(non_snake_case)]
    for i in 0..64 {
        // calculate S1
        let (a1, extra_gates, updated_witness_counter) = e.ror(6, num_witness);
        new_gates.extend(extra_gates);
        let (a2, extra_gates, updated_witness_counter) = e.ror(11, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (a3, extra_gates, updated_witness_counter) = e.ror(25, updated_witness_counter);
        new_gates.extend(extra_gates);

        let (a4, extra_gates, updated_witness_counter) = a1.xor(a2, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (S1, extra_gates, updated_witness_counter) = a4.xor(a3, updated_witness_counter);
        new_gates.extend(extra_gates);

        // calculate ch
        let (b1, extra_gates, updated_witness_counter) = e.and(f.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);
        let (b2, extra_gates, updated_witness_counter) = e.not(updated_witness_counter);
        new_gates.extend(extra_gates);
        let (b3, extra_gates, updated_witness_counter) = b2.and(g.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);

        let (ch, extra_gates, updated_witness_counter) = b1.add(b3, updated_witness_counter);
        new_gates.extend(extra_gates);

        // caculate temp1
        let (c1, extra_gates, updated_witness_counter) = h.add(S1.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);
        let (c2, extra_gates, updated_witness_counter) =
            c1.add(ch.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);
        let (c3, extra_gates, updated_witness_counter) =
            c2.add(round_constants[i].clone(), updated_witness_counter);
        new_gates.extend(extra_gates);

        let (temp1, extra_gates, updated_witness_counter) =
            c3.add(w[i].clone(), updated_witness_counter);
        new_gates.extend(extra_gates);

        // calculate S0
        let (d1, extra_gates, updated_witness_counter) = a.ror(2, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (d2, extra_gates, updated_witness_counter) = a.ror(13, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (d3, extra_gates, updated_witness_counter) = a.ror(22, updated_witness_counter);
        new_gates.extend(extra_gates);

        let (d4, extra_gates, updated_witness_counter) = d1.xor(d2, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (S0, extra_gates, updated_witness_counter) = d4.xor(d3, updated_witness_counter);
        new_gates.extend(extra_gates);

        // calculate T0
        let (T0, extra_gates, updated_witness_counter) = b.and(c.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);

        // calculate maj
        let (e1, extra_gates, updated_witness_counter) =
            T0.add(T0.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);
        let (e2, extra_gates, updated_witness_counter) = c.sub(e1, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (e3, extra_gates, updated_witness_counter) = b.add(e2, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (e4, extra_gates, updated_witness_counter) = a.and(e3, updated_witness_counter);
        new_gates.extend(extra_gates);
        let (maj, extra_gates, updated_witness_counter) =
            e4.add(T0.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);

        // calculate temp2
        let (temp2, extra_gates, updated_witness_counter) =
            S0.add(maj.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);

        h = g;
        g = f;
        f = e.clone();
        let (new_e, extra_gates, updated_witness_counter) =
            d.add(temp1.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);
        d = c;
        c = b;
        b = a.clone();
        let (new_a, extra_gates, updated_witness_counter) =
            temp1.add(temp2.clone(), updated_witness_counter);
        new_gates.extend(extra_gates);

        num_witness = updated_witness_counter;
        a = new_a;
        e = new_e;
    }

    let mut output = Vec::new();
    let (output0, extra_gates, num_witness) = a.add(rolling_hash[0].clone(), num_witness);
    new_gates.extend(extra_gates);
    let (output1, extra_gates, num_witness) = b.add(rolling_hash[1].clone(), num_witness);
    new_gates.extend(extra_gates);
    let (output2, extra_gates, num_witness) = c.add(rolling_hash[2].clone(), num_witness);
    new_gates.extend(extra_gates);
    let (output3, extra_gates, num_witness) = d.add(rolling_hash[3].clone(), num_witness);
    new_gates.extend(extra_gates);
    let (output4, extra_gates, num_witness) = e.add(rolling_hash[4].clone(), num_witness);
    new_gates.extend(extra_gates);
    let (output5, extra_gates, num_witness) = f.add(rolling_hash[5].clone(), num_witness);
    new_gates.extend(extra_gates);
    let (output6, extra_gates, num_witness) = g.add(rolling_hash[6].clone(), num_witness);
    new_gates.extend(extra_gates);
    let (output7, extra_gates, num_witness) = h.add(rolling_hash[7].clone(), num_witness);
    new_gates.extend(extra_gates);

    output.push(output0);
    output.push(output1);
    output.push(output2);
    output.push(output3);
    output.push(output4);
    output.push(output5);
    output.push(output6);
    output.push(output7);

    (output, new_gates, num_witness)
}
