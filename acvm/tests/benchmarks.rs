use acir::circuit::{Circuit, Opcode};
use criterion::{criterion_group, criterion_main, Criterion};

use acvm::{compiler::compile, *};

fn compile_bench(c: &mut Criterion) {
    c.bench_function("compile_bench", |b| {
        b.iter(|| compile(Circuit::default(), Language::R1CS, |_: &Opcode| true))
    });
}

criterion_group! {
  name = benches;
  config = Criterion::default().sample_size(10);
  targets =
    compile_bench
}

criterion_main!(benches);
