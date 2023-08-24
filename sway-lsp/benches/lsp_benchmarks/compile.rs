use criterion::{black_box, criterion_group, Criterion};

fn benchmarks(c: &mut Criterion) {
    c.bench_function("compile_and_traverse", |b| {
        b.iter(|| {
            let _ = black_box(super::compile_test_project());
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = benchmarks
}
