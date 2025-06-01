use codspeed_criterion_compat::{black_box, criterion_group, Criterion};
use lsp_types::Position;
use tokio::runtime::Runtime;

fn benchmarks(c: &mut Criterion) {
    let (uri, _, state, engines) = Runtime::new()
        .unwrap()
        .block_on(async { black_box(super::compile_test_project().await) });
    let sync = state.get_sync_workspace_for_uri(&uri).unwrap();
    let position = Position::new(1716, 24);
    let path = uri.to_file_path().unwrap();
    let program_id = sway_lsp::core::session::program_id_from_path(&path, &engines).unwrap();
    c.bench_function("tokens_for_program", |b| {
        b.iter(|| {
            let _: Vec<_> = state.token_map.tokens_for_program(program_id).collect();
        })
    });

    c.bench_function("tokens_for_file", |b| {
        b.iter(|| {
            let _: Vec<_> = state.token_map.tokens_for_file(&uri).collect();
        })
    });

    c.bench_function("idents_at_position", |b| {
        b.iter(|| {
            state
                .token_map
                .idents_at_position(position, state.token_map.iter())
        })
    });

    c.bench_function("tokens_at_position", |b| {
        b.iter(|| {
            state
                .token_map
                .tokens_at_position(&engines, &uri, position, None)
        })
    });

    c.bench_function("token_at_position", |b| {
        b.iter(|| state.token_map.token_at_position(&uri, position))
    });

    c.bench_function("parent_decl_at_position", |b| {
        b.iter(|| {
            state
                .token_map
                .parent_decl_at_position(&engines, &uri, position)
        })
    });

    // Remove the temp dir after the benchmarks are done
    Runtime::new()
        .unwrap()
        .block_on(async { sync.remove_temp_dir() });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = benchmarks
}
