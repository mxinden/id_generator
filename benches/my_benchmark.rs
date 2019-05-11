#[macro_use]
extern crate criterion;
extern crate id_generator;

use criterion::{Criterion, ParameterizedBenchmark};
use id_generator::Generator;

fn criterion_benchmark(c: &mut Criterion) {
    let mut g= id_generator::BasicGenerator::new(0,id_generator::time::Clock::new());

    c.bench_function("generate", move |b| b.iter(|| g.generate()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
