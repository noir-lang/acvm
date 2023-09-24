use acir_field::FieldElement;
use criterion::{criterion_group, criterion_main, Criterion};
use ruint::{Uint, aliases::U256};

const BN254_MODULUS: Uint<256, 4> = ruint::uint!(21888242871839275222246405745257275088548364400416034343698204186575808495617_U256);

fn pow_bench(c: &mut Criterion) {
    let x = FieldElement::from(100 as i128);
    let y = FieldElement::from(100 as i128);

    c.bench_function("pow_bench", |b| b.iter(|| x.pow(&y)));
}

fn try_from_str_bench(c: &mut Criterion) {
    let x = "100";

    c.bench_function("try_from_str_bench", |b| {
        b.iter(|| FieldElement::try_from_str(x));
    });
}

fn num_bits_bench(c: &mut Criterion) {
    let x = FieldElement::from(100 as i128);

    c.bench_function("num_bits_bench", |b| {
        b.iter(|| x.num_bits());
    });
}

fn to_u128_bench(c: &mut Criterion) {
    let x = FieldElement::from(100000000000 as i128);

    c.bench_function("to_u128_bench", |b| {
        b.iter(|| x.to_u128());
    });
}

fn inverse_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);

    c.bench_function("inverse_bench", |b| {
        b.iter(|| x.inverse());
    });
}

fn to_hex_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);

    c.bench_function("to_hex_bench", |b| {
        b.iter(|| x.to_hex());
    });
}

fn from_hex_bench(c: &mut Criterion) {
    let x = "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000";

    c.bench_function("from_hex_bench", |b| {
        b.iter(|| FieldElement::from_hex(x));
    });
}

fn to_be_bytes_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);

    c.bench_function("to_be_bytes_bench", |b| {
        b.iter(|| x.to_be_bytes());
    });
}

fn bits_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);

    c.bench_function("bits_bench", |b| {
        b.iter(|| x.bits());
    });
}

fn fetch_nearest_bytes_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);

    c.bench_function("fetch_nearest_bytes_bench", |b| {
        b.iter(|| x.fetch_nearest_bytes(10));
    });
}

fn and_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);
    let y = FieldElement::from(77777 as i128);

    c.bench_function("and_bench", |b| {
        b.iter(|| x.and(&y, 10));
    });
}

fn xor_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);
    let y = FieldElement::from(77777 as i128);

    c.bench_function("xor_bench", |b| {
        b.iter(|| x.xor(&y, 10));
    });
}

fn add_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);
    let y = FieldElement::from(77777 as i128);

    c.bench_function("add_bench", |b| {
        b.iter(|| x + y);
    });
}

fn sub_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);
    let y = FieldElement::from(77777 as i128);

    c.bench_function("sub_bench", |b| {
        b.iter(|| x - y);
    });
}

fn mul_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);
    let y = FieldElement::from(77777 as i128);

    c.bench_function("mul_bench", |b| {
        b.iter(|| x * y);
    });
}

fn div_bench(c: &mut Criterion) {
    let x = FieldElement::from(10000 as i128);
    let y = FieldElement::from(77777 as i128);

    c.bench_function("div_bench", |b| {
        b.iter(|| x / y);
    });
}

criterion_group! {
  name = ark_bn254_benches;
  config = Criterion::default().sample_size(10);
  targets =
    pow_bench, try_from_str_bench, num_bits_bench, to_u128_bench,
    inverse_bench, to_hex_bench, from_hex_bench, to_be_bytes_bench,
    bits_bench, fetch_nearest_bytes_bench, and_bench, xor_bench,
    add_bench, sub_bench, mul_bench, div_bench
}

fn add_uint_bench(c: &mut Criterion) {
    let x: Uint<256, 4> = Uint::from(10000);
    let y: Uint<256, 4> = Uint::from(77777);

    c.bench_function("add_uint_bench", |b| {
        b.iter(|| (x + y) % BN254_MODULUS);
    });
}

fn sub_uint_bench(c: &mut Criterion) {
    let x: Uint<256, 4> = Uint::from(10000);
    let y: Uint<256, 4> = Uint::from(77777);

    c.bench_function("sub_uint_bench", |b| {
        b.iter(|| (x - y) % BN254_MODULUS);
    });
}

fn mul_uint_bench(c: &mut Criterion) {
    let x: Uint<256, 4> = Uint::from(10000);
    let y: Uint<256, 4> = Uint::from(77777);

    c.bench_function("mul_uint_bench", |b| {
        b.iter(|| (x * y) % BN254_MODULUS);
    });
}

fn div_uint_bench(c: &mut Criterion) {
    let x: Uint<256, 4> = Uint::from(10000);
    let y: Uint<256, 4> = Uint::from(77777);

    c.bench_function("div_uint_bench", |b| {
        b.iter(|| (x / y) % BN254_MODULUS);
    });
}

criterion_group! {
    name = uint_benches;
    config = Criterion::default().sample_size(10);
    targets =
        add_uint_bench, sub_uint_bench, mul_uint_bench, div_uint_bench
}

fn from_div_bench(c: &mut Criterion) {
    let x: Uint<254, 4> = Uint::from(10000);
    let y: Uint<254, 4> = Uint::from(77777);

    let x1 = FieldElement::from(x);
    let y1 = FieldElement::from(y);

    c.bench_function("from_div_bench", |b| {
        b.iter(|| x1 / y1);
    });
}

fn from_mul_bench(c: &mut Criterion) {
    let x: Uint<254, 4> = Uint::from(10000);
    let y: Uint<254, 4> = Uint::from(77777);

    let x1 = FieldElement::from(x);
    let y1 = FieldElement::from(y);

    c.bench_function("from_mul_bench", |b| {
        b.iter(|| x1 * y1);
    });
}

fn from_add_bench(c: &mut Criterion) {
    let x: Uint<254, 4> = Uint::from(10000);
    let y: Uint<254, 4> = Uint::from(77777);

    let x1 = FieldElement::from(x);
    let y1 = FieldElement::from(y);

    c.bench_function("from_add_bench", |b| {
        b.iter(|| x1 + y1);
    });
}

fn from_sub_bench(c: &mut Criterion) {
    let x: Uint<254, 4> = Uint::from(10000);
    let y: Uint<254, 4> = Uint::from(77777);

    let x1 = FieldElement::from(x);
    let y1 = FieldElement::from(y);

    c.bench_function("from_sub_bench", |b| {
        b.iter(|| x1 - y1);
    });
}

criterion_group! {
    name = from_benches;
    config = Criterion::default().sample_size(10);
    targets =
        from_div_bench, from_mul_bench, from_add_bench, from_sub_bench
}

criterion_main!(ark_bn254_benches, uint_benches, from_benches);
