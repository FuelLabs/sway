use criterion::{black_box, criterion_group, Criterion};
use lsp_types::Position;
use sway_lsp::{capabilities, utils::keyword_docs::KeywordDocs};

fn benchmarks(c: &mut Criterion) {
    let (uri, session) = black_box(super::compile_test_project());
    let keyword_docs = KeywordDocs::new();
    let position = Position::new(1716, 24);

    c.bench_function("hover", |b| {
        b.iter(|| {
            capabilities::hover::hover_data(session.clone(), &keyword_docs, uri.clone(), position)
        })
    });

    c.bench_function("highlight", |b| {
        b.iter(|| capabilities::highlight::get_highlights(session.clone(), uri.clone(), position))
    });

    c.bench_function("goto_definition", |b| {
        b.iter(|| session.token_definition_response(uri.clone(), position))
    });

    c.bench_function("prepare_rename", |b| {
        b.iter(|| capabilities::rename::prepare_rename(session.clone(), uri.clone(), position))
    });

    c.bench_function("rename", |b| {
        b.iter(|| {
            capabilities::rename::rename(
                session.clone(),
                "new_token_name".to_string(),
                uri.clone(),
                position,
            )
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = benchmarks
}
