use criterion::{Criterion, criterion_group, criterion_main};

fn basic_bench(c: &mut Criterion) {
    c.bench_function("dummy_bench", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, basic_bench);
criterion_main!(benches);
