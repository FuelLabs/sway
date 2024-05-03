use criterion::{black_box, criterion_group, Criterion};
use lsp_types::{
    CompletionResponse, DocumentSymbolResponse, Position, Range, TextDocumentContentChangeEvent,
    TextDocumentIdentifier,
};
use sway_lsp::{capabilities, lsp_ext::OnEnterParams, utils::keyword_docs::KeywordDocs};
use tokio::runtime::Runtime;

fn benchmarks(c: &mut Criterion) {
    let (uri, session, documents) = Runtime::new()
        .unwrap()
        .block_on(async { black_box(super::compile_test_project().await) });
    let config = sway_lsp::config::Config::default();
    let keyword_docs = KeywordDocs::new();
    let position = Position::new(1717, 24);
    let range = Range::new(Position::new(1628, 0), Position::new(1728, 0));

    c.bench_function("semantic_tokens", |b| {
        b.iter(|| capabilities::semantic_tokens::semantic_tokens_full(session.clone(), &uri))
    });

    c.bench_function("document_symbol", |b| {
        b.iter(|| {
            session
                .symbol_information(&uri)
                .map(DocumentSymbolResponse::Flat)
        })
    });

    c.bench_function("completion", |b| {
        let position = Position::new(1698, 28);
        b.iter(|| {
            session
                .completion_items(&uri, position, ".")
                .map(CompletionResponse::Array)
        })
    });

    c.bench_function("hover", |b| {
        b.iter(|| capabilities::hover::hover_data(session.clone(), &keyword_docs, &uri, position))
    });

    c.bench_function("highlight", |b| {
        b.iter(|| capabilities::highlight::get_highlights(session.clone(), &uri, position))
    });

    c.bench_function("goto_definition", |b| {
        b.iter(|| session.token_definition_response(&uri, position))
    });

    c.bench_function("inlay_hints", |b| {
        b.iter(|| {
            capabilities::inlay_hints::inlay_hints(
                session.clone(),
                &uri,
                &range,
                &config.inlay_hints,
            )
        })
    });

    c.bench_function("prepare_rename", |b| {
        b.iter(|| capabilities::rename::prepare_rename(session.clone(), &uri, position))
    });

    c.bench_function("rename", |b| {
        b.iter(|| {
            capabilities::rename::rename(
                session.clone(),
                "new_token_name".to_string(),
                &uri,
                position,
            )
        })
    });

    c.bench_function("code_action", |b| {
        let range = Range::new(Position::new(4, 10), Position::new(4, 10));
        b.iter(|| {
            capabilities::code_actions::code_actions(session.clone(), &range, &uri, &uri, &vec![])
        })
    });

    c.bench_function("code_lens", |b| {
        b.iter(|| capabilities::code_lens::code_lens(&session, &uri.clone()))
    });

    c.bench_function("on_enter", |b| {
        let params = OnEnterParams {
            text_document: TextDocumentIdentifier::new(uri.clone()),
            content_changes: vec![TextDocumentContentChangeEvent {
                range: Some(Range::new(Position::new(3, 30), Position::new(3, 30))),
                range_length: Some(0),
                text: "\n".to_string(),
            }],
        };
        b.iter(|| capabilities::on_enter::on_enter(&config.on_enter, &documents, &uri, &params))
    });

    c.bench_function("format", |b| {
        b.iter(|| capabilities::formatting::format_text(&documents, &uri))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(3));
    targets = benchmarks
}
