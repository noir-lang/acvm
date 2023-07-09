//! Blake2s fallback function.
use std::vec;

use super::utils::{byte_decomposition, round_to_nearest_byte};
pub use super::wu32::WU32;

use acir::{
    circuit::Opcode,
    native_types::{Expression, Witness},
    FieldElement,
};

const BLAKE2S_BLOCKBYTES_USIZE: usize = 64;
const MSG_SCHEDULE_BLAKE2: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

pub fn blake2s(
    inputs: Vec<(Expression, u32)>,
    outputs: Vec<Witness>,
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

    // constrain the outputs to be the same as the result of the circuit
    for i in 0..outputs.len() {
        let mut expr = Expression::from(outputs[i]);
        expr.push_addition_term(-FieldElement::one(), result[i]);
        new_gates.push(Opcode::Arithmetic(expr));
    }
    (num_witness, new_gates)
}

fn create_blake2s_constraint(
    input: Vec<Witness>,
    num_witness: u32,
) -> (Vec<Witness>, u32, Vec<Opcode>) {
    let mut new_gates = Vec::new();

    // prepare constants
    let (mut blake2s_state, extra_gates, num_witness) = Blake2sState::init(num_witness);
    new_gates.extend(extra_gates);
    let (blake2s_constants, extra_gates, num_witness) =
        Blake2sConstantsInCircuit::init(num_witness);
    new_gates.extend(extra_gates);
    let (blake2s_iv, extra_gates, mut num_witness) = Blake2sIV::init(num_witness);
    new_gates.extend(extra_gates);

    let mut offset = 0;
    let mut size = input.len();

    while size > BLAKE2S_BLOCKBYTES_USIZE {
        let (extra_gates, updated_witness_counter) = blake2s_increment_counter(
            &mut blake2s_state,
            &blake2s_constants.blake2s_blockbytes_wu32,
            num_witness,
        );
        new_gates.extend(extra_gates);
        let (extra_gates, updated_witness_counter) = blake2s_compress(
            &mut blake2s_state,
            &blake2s_iv,
            input.get(offset..offset + BLAKE2S_BLOCKBYTES_USIZE).unwrap(),
            updated_witness_counter,
        );
        new_gates.extend(extra_gates);
        offset += BLAKE2S_BLOCKBYTES_USIZE;
        size -= BLAKE2S_BLOCKBYTES_USIZE;
        num_witness = updated_witness_counter;
    }

    let (u32_max, extra_gates, mut num_witness) =
        WU32::load_constant(u32::MAX as u128, num_witness);
    new_gates.extend(extra_gates);
    blake2s_state.f[0] = u32_max;

    // pad final block
    let mut final_block = input.get(offset..).unwrap().to_vec();
    for _ in 0..BLAKE2S_BLOCKBYTES_USIZE - final_block.len() {
        let (pad, extra_gates, updated_witness_counter) = WU32::load_constant(0_u128, num_witness);
        new_gates.extend(extra_gates);
        final_block.push(pad.inner);
        num_witness = updated_witness_counter;
    }

    let (size_w, extra_gates, num_witness) = WU32::load_constant(size as u128, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) =
        blake2s_increment_counter(&mut blake2s_state, &size_w, num_witness);
    new_gates.extend(extra_gates);

    let (extra_gates, num_witness) =
        blake2s_compress(&mut blake2s_state, &blake2s_iv, &final_block, num_witness);
    new_gates.extend(extra_gates);

    // decompose the result bytes in u32 to u8
    let (extra_gates, mut byte1, num_witness) =
        byte_decomposition(Expression::from(blake2s_state.h[0].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, mut byte2, num_witness) =
        byte_decomposition(Expression::from(blake2s_state.h[1].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, mut byte3, num_witness) =
        byte_decomposition(Expression::from(blake2s_state.h[2].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, mut byte4, num_witness) =
        byte_decomposition(Expression::from(blake2s_state.h[3].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, mut byte5, num_witness) =
        byte_decomposition(Expression::from(blake2s_state.h[4].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, mut byte6, num_witness) =
        byte_decomposition(Expression::from(blake2s_state.h[5].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, mut byte7, num_witness) =
        byte_decomposition(Expression::from(blake2s_state.h[6].inner), 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, mut byte8, num_witness) =
        byte_decomposition(Expression::from(blake2s_state.h[7].inner), 4, num_witness);
    new_gates.extend(extra_gates);

    byte1.reverse();
    byte2.reverse();
    byte3.reverse();
    byte4.reverse();
    byte5.reverse();
    byte6.reverse();
    byte7.reverse();
    byte8.reverse();

    let result = vec![byte1, byte2, byte3, byte4, byte5, byte6, byte7, byte8]
        .into_iter()
        .flatten()
        .collect();

    (result, num_witness, new_gates)
}

fn blake2s_increment_counter(
    mut state: &mut Blake2sState,
    inc: &WU32,
    num_witness: u32,
) -> (Vec<Opcode>, u32) {
    let mut new_gates = Vec::new();

    // t0 + inc
    let (state_t0, extra_gates, num_witness) = state.t[0].add(inc, num_witness);
    new_gates.extend(extra_gates);
    state.t[0] = state_t0;

    // t1 + (t0 < inc)
    let (to_inc, extra_gates, num_witness) = state.t[0].less_than_comparison(inc, num_witness);
    new_gates.extend(extra_gates);
    let (state_t1, extra_gates, num_witness) = state.t[1].add(&to_inc, num_witness);
    new_gates.extend(extra_gates);
    state.t[1] = state_t1;

    (new_gates, num_witness)
}

fn blake2s_compress(
    mut state: &mut Blake2sState,
    blake2s_iv: &Blake2sIV,
    input: &[Witness],
    mut num_witness: u32,
) -> (Vec<Opcode>, u32) {
    let mut new_gates = Vec::new();
    let mut m = Vec::new();
    let mut v = Vec::new();

    for i in 0..16 {
        let mut mi_bytes = input.get(i * 4..i * 4 + 4).unwrap().to_vec();
        mi_bytes.reverse();
        let (mi, extra_gates, updated_witness_counter) =
            WU32::from_witnesses(&mi_bytes, num_witness);
        new_gates.extend(extra_gates);
        m.push(mi[0]);
        num_witness = updated_witness_counter
    }

    for i in 0..8 {
        v.push(state.h[i]);
    }

    v.push(blake2s_iv.iv[0]);
    v.push(blake2s_iv.iv[1]);
    v.push(blake2s_iv.iv[2]);
    v.push(blake2s_iv.iv[3]);
    let (v12, extra_gates, num_witness) = state.t[0].xor(&blake2s_iv.iv[4], num_witness);
    new_gates.extend(extra_gates);
    v.push(v12);
    let (v13, extra_gates, num_witness) = state.t[1].xor(&blake2s_iv.iv[5], num_witness);
    new_gates.extend(extra_gates);
    v.push(v13);
    let (v14, extra_gates, num_witness) = state.f[0].xor(&blake2s_iv.iv[6], num_witness);
    new_gates.extend(extra_gates);
    v.push(v14);
    let (v15, extra_gates, num_witness) = state.f[1].xor(&blake2s_iv.iv[7], num_witness);
    new_gates.extend(extra_gates);
    v.push(v15);

    let (extra_gates, num_witness) = blake2s_round(&mut v, &m, 0, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) = blake2s_round(&mut v, &m, 1, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) = blake2s_round(&mut v, &m, 2, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) = blake2s_round(&mut v, &m, 3, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) = blake2s_round(&mut v, &m, 4, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) = blake2s_round(&mut v, &m, 5, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) = blake2s_round(&mut v, &m, 6, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) = blake2s_round(&mut v, &m, 7, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) = blake2s_round(&mut v, &m, 8, num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, mut num_witness) = blake2s_round(&mut v, &m, 9, num_witness);
    new_gates.extend(extra_gates);

    for i in 0..8 {
        let (a, extra_gates, updated_witness_counter) = state.h[i].xor(&v[i], num_witness);
        new_gates.extend(extra_gates);
        let (state_hi, extra_gates, updated_witness_counter) =
            a.xor(&v[i + 8], updated_witness_counter);
        new_gates.extend(extra_gates);
        state.h[i] = state_hi;
        num_witness = updated_witness_counter;
    }

    (new_gates, num_witness)
}

fn blake2s_round(
    state: &mut [WU32],
    msg: &[WU32],
    round: usize,
    num_witness: u32,
) -> (Vec<Opcode>, u32) {
    let mut new_gates = Vec::new();
    let schedule = &MSG_SCHEDULE_BLAKE2[round];

    // Mix the columns.
    let (extra_gates, num_witness) =
        g(state, 0, 4, 8, 12, msg[schedule[0]], msg[schedule[1]], num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) =
        g(state, 1, 5, 9, 13, msg[schedule[2]], msg[schedule[3]], num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) =
        g(state, 2, 6, 10, 14, msg[schedule[4]], msg[schedule[5]], num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) =
        g(state, 3, 7, 11, 15, msg[schedule[6]], msg[schedule[7]], num_witness);
    new_gates.extend(extra_gates);

    // Mix the rows.
    let (extra_gates, num_witness) =
        g(state, 0, 5, 10, 15, msg[schedule[8]], msg[schedule[9]], num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) =
        g(state, 1, 6, 11, 12, msg[schedule[10]], msg[schedule[11]], num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) =
        g(state, 2, 7, 8, 13, msg[schedule[12]], msg[schedule[13]], num_witness);
    new_gates.extend(extra_gates);
    let (extra_gates, num_witness) =
        g(state, 3, 4, 9, 14, msg[schedule[14]], msg[schedule[15]], num_witness);
    new_gates.extend(extra_gates);

    (new_gates, num_witness)
}

#[allow(clippy::too_many_arguments)]
fn g(
    state: &mut [WU32],
    a: usize,
    b: usize,
    c: usize,
    d: usize,
    x: WU32,
    y: WU32,
    num_witness: u32,
) -> (Vec<Opcode>, u32) {
    let mut new_gates = Vec::new();

    // calculate state[a] as `state[a] + state[b] + x`
    let (state_a_1, extra_gates, num_witness) = state[a].add(&state[b], num_witness);
    new_gates.extend(extra_gates);
    let (state_a, extra_gates, num_witness) = state_a_1.add(&x, num_witness);
    new_gates.extend(extra_gates);
    state[a] = state_a;

    // calculate state[d] as `(state[d] ^ state[a]).ror(16)`
    let (state_d_1, extra_gates, num_witness) = state[d].xor(&state[a], num_witness);
    new_gates.extend(extra_gates);
    let (state_d, extra_gates, num_witness) = state_d_1.ror(16, num_witness);
    new_gates.extend(extra_gates);
    state[d] = state_d;

    // calculate state[c] as `state[c] + state[d]`
    let (state_c, extra_gates, num_witness) = state[c].add(&state[d], num_witness);
    new_gates.extend(extra_gates);
    state[c] = state_c;

    // caclulate state[b] as `(state[b] ^ state[c]).ror(12)`
    let (state_b_1, extra_gates, num_witness) = state[b].xor(&state[c], num_witness);
    new_gates.extend(extra_gates);
    let (state_b, extra_gates, num_witness) = state_b_1.ror(12, num_witness);
    new_gates.extend(extra_gates);
    state[b] = state_b;

    // calculate state[a] as `state[a] + state[b] + y`
    let (state_a_1, extra_gates, num_witness) = state[a].add(&state[b], num_witness);
    new_gates.extend(extra_gates);
    let (state_a, extra_gates, num_witness) = state_a_1.add(&y, num_witness);
    new_gates.extend(extra_gates);
    state[a] = state_a;

    // calculate state[d] as `(state[d] ^ state[a]).ror(8)`
    let (state_d_1, extra_gates, num_witness) = state[d].xor(&state[a], num_witness);
    new_gates.extend(extra_gates);
    let (state_d, extra_gates, num_witness) = state_d_1.ror(8, num_witness);
    new_gates.extend(extra_gates);
    state[d] = state_d;

    // calculate state[c] as `state[c] + state[d]`
    let (state_c, extra_gates, num_witness) = state[c].add(&state[d], num_witness);
    new_gates.extend(extra_gates);
    state[c] = state_c;

    // caclulate state[b] as `(state[b] ^ state[c]).ror(7)`
    let (state_b_1, extra_gates, num_witness) = state[b].xor(&state[c], num_witness);
    new_gates.extend(extra_gates);
    let (state_b, extra_gates, num_witness) = state_b_1.ror(7, num_witness);
    new_gates.extend(extra_gates);
    state[b] = state_b;

    (new_gates, num_witness)
}

/// Blake2s state `h` `t` and `f`
#[derive(Debug)]
struct Blake2sState {
    h: [WU32; 8],
    t: [WU32; 2],
    f: [WU32; 2],
}

impl Blake2sState {
    fn new(h: [WU32; 8], t: [WU32; 2], f: [WU32; 2]) -> Self {
        Blake2sState { h, t, f }
    }

    /// Initialize internal state of Blake2s
    fn init(mut num_witness: u32) -> (Blake2sState, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut h = [WU32::default(); 8];
        let mut t = [WU32::default(); 2];
        let mut f = [WU32::default(); 2];
        let initial_h: Vec<u128> = vec![
            0x6b08e647, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
            0x5be0cd19,
        ];

        for i in 0..initial_h.len() {
            let (new_witness, extra_gates, updated_witness_counter) =
                WU32::load_constant(initial_h[i], num_witness);
            new_gates.extend(extra_gates);
            h[i] = new_witness;
            num_witness = updated_witness_counter;
        }

        for ti in &mut t {
            let (new_witness, extra_gates, updated_witness_counter) =
                WU32::load_constant(0_u128, num_witness);
            new_gates.extend(extra_gates);
            *ti = new_witness;
            num_witness = updated_witness_counter;
        }

        for fi in &mut f {
            let (new_witness, extra_gates, updated_witness_counter) =
                WU32::load_constant(0_u128, num_witness);
            new_gates.extend(extra_gates);
            *fi = new_witness;
            num_witness = updated_witness_counter;
        }

        let blake2s_state = Blake2sState::new(h, t, f);

        (blake2s_state, new_gates, num_witness)
    }
}

/// Blake2s IV (Initialization Vector)
struct Blake2sIV {
    iv: [WU32; 8],
}

impl Blake2sIV {
    fn new(iv: [WU32; 8]) -> Self {
        Blake2sIV { iv }
    }

    /// Initialize IV of Blake2s
    fn init(mut num_witness: u32) -> (Blake2sIV, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let mut iv = [WU32::default(); 8];

        let iv_value: Vec<u128> = vec![
            0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB,
            0x5BE0CD19,
        ];

        for i in 0..iv_value.len() {
            let (new_witness, extra_gates, updated_witness_counter) =
                WU32::load_constant(iv_value[i], num_witness);
            new_gates.extend(extra_gates);
            iv[i] = new_witness;
            num_witness = updated_witness_counter;
        }

        let blake2s_iv = Blake2sIV::new(iv);

        (blake2s_iv, new_gates, num_witness)
    }
}

struct Blake2sConstantsInCircuit {
    blake2s_blockbytes_wu32: WU32,
}

impl Blake2sConstantsInCircuit {
    fn new(blake2s_blockbytes_wu32: WU32) -> Self {
        Blake2sConstantsInCircuit { blake2s_blockbytes_wu32 }
    }

    fn init(num_witness: u32) -> (Blake2sConstantsInCircuit, Vec<Opcode>, u32) {
        let mut new_gates = Vec::new();
        let (blake2s_blockbytes_wu32, extra_gates, num_witness) =
            WU32::load_constant(64_u128, num_witness);
        new_gates.extend(extra_gates);

        (Blake2sConstantsInCircuit::new(blake2s_blockbytes_wu32), new_gates, num_witness)
    }
}
