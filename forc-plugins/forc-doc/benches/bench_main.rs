use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};
use forc_doc::{compile, compile_html, Command, DocContext};
use std::path::Path;

fn benchmarks(c: &mut Criterion) {
    let path = Path::new("./../../sway-lib-std");
    let opts = Command {
        path: Some(path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    let ctx = DocContext::from_options(&opts).unwrap();
    let compile_results = compile(&ctx, &opts).unwrap().collect::<Vec<_>>();

    c.bench_function("build_std_lib_docs", |b| {
        b.iter(|| {
            let mut results = compile_results.clone();
            let _ = compile_html(&opts, &ctx, &mut results);
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = benchmarks
}

criterion_main!(benches);
