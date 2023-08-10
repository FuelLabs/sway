use criterion::{black_box, criterion_group, Criterion};
use lsp_types::Position;

fn benchmarks(c: &mut Criterion) {
    let (uri, session) = black_box(super::compile_test_project());
    let engines = session.engines.read();
    let position = Position::new(1716, 24);

    c.bench_function("tokens_for_file", |b| {
        b.iter(|| {
            let _: Vec<_> = session
                .token_map()
                .tokens_for_file(engines.se(), &uri)
                .collect();
        })
    });

    c.bench_function("idents_at_position", |b| {
        b.iter(|| {
            session
                .token_map()
                .idents_at_position(position, session.token_map().iter())
        })
    });

    c.bench_function("tokens_at_position", |b| {
        b.iter(|| {
            session
                .token_map()
                .tokens_at_position(engines.se(), &uri, position, None)
        })
    });

    c.bench_function("token_at_position", |b| {
        b.iter(|| {
            session
                .token_map()
                .token_at_position(engines.se(), &uri, position)
        })
    });

    c.bench_function("parent_decl_at_position", |b| {
        b.iter(|| {
            session
                .token_map()
                .parent_decl_at_position(engines.se(), &uri, position)
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = benchmarks
}
