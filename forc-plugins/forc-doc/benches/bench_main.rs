use codspeed_criterion_compat::{
    Criterion,
    criterion_group,
    criterion_main,
};

fn benchmarks(c: &mut Criterion) {


    c.bench_function("add_market", |b| {
        b.iter(|| {
            // TODO: Implement benchmarks
        });
    });

}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = benchmarks
}

criterion_main!(benches);