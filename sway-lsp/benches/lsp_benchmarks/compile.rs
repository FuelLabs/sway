use criterion::{black_box, criterion_group, Criterion};
use lsp_types::Url;
use sway_core::Engines;
use sway_lsp::core::session;

const NUM_DID_CHANGE_ITERATIONS: usize = 10;

fn benchmarks(c: &mut Criterion) {
    // Load the test project
    let uri = Url::from_file_path(super::benchmark_dir().join("src/main.sw")).unwrap();
    c.bench_function("compile", |b| {
        b.iter(|| {
            let engines = Engines::default();
            let _ = black_box(session::compile(&uri, None, &engines, None).unwrap());
        })
    });

    c.bench_function("traverse", |b| {
        let engines = Engines::default();
        let results = black_box(session::compile(&uri, None, &engines, None).unwrap());
        b.iter(|| {
            let _ = black_box(session::traverse(results.clone(), &engines).unwrap());
        })
    });

    c.bench_function("did_change_with_caching", |b| {
        let engines = Engines::default();
        b.iter(|| {
            for version in 0..NUM_DID_CHANGE_ITERATIONS {
                let _ = black_box(session::compile(&uri, Some(version as i32), &engines, None).unwrap());
            }
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10)).sample_size(10);
    targets = benchmarks
}
