#[macro_use]
extern crate criterion;
extern crate id_generator;

use criterion::{Criterion, ParameterizedBenchmark};

fn criterion_benchmark(c: &mut Criterion) {
    let mut g32 = id_generator::Generator::<i32>::new(0,1);
    let mut g64 = id_generator::Generator::<i64>::new(0,1);
    let mut g128 = id_generator::Generator::<i128>::new(0,1);

    c.bench("generate",
            ParameterizedBenchmark::new(
                "i32", move|b, i| b.iter( || g32.generate()), vec![0] ).with_function(
                "i64", move|b, i| b.iter( ||g64.generate())).with_function(
                "i128", move|b, i| b.iter( ||g128.generate())
            )
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
