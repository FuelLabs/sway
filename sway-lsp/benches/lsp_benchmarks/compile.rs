use codspeed_criterion_compat::{black_box, criterion_group, Criterion};
use std::sync::Arc;
use sway_core::Engines;
use sway_lsp::core::session;
use tokio::runtime::Runtime;

const NUM_DID_CHANGE_ITERATIONS: usize = 10;

fn benchmarks(c: &mut Criterion) {
    let (uri, session, state) = Runtime::new()
        .unwrap()
        .block_on(async { black_box(super::compile_test_project().await) });

    let sync = state.sync_workspace.get().unwrap();
    let build_plan = session
        .build_plan_cache
        .get_or_update(&sync.workspace_manifest_path(), || {
            session::build_plan(&uri)
        })
        .unwrap();

    let mut lsp_mode = Some(sway_core::LspConfig {
        optimized_build: false,
        file_versions: Default::default(),
    });

    c.bench_function("compile", |b| {
        b.iter(|| {
            let engines = Engines::default();
            let _ = black_box(
                session::compile(&build_plan, &engines, None, lsp_mode.as_ref()).unwrap(),
            );
        })
    });

    c.bench_function("traverse", |b| {
        let engines = Engines::default();
        let results =
            black_box(session::compile(&build_plan, &engines, None, lsp_mode.as_ref()).unwrap());
        let session = Arc::new(session::Session::new());
        let member_path = sync.member_path(&uri).unwrap();

        b.iter(|| {
            let _ = black_box(
                session::traverse(
                    member_path.clone(),
                    results.clone(),
                    &engines,
                    session.clone(),
                    lsp_mode.as_ref(),
                )
                .unwrap(),
            );
        })
    });

    lsp_mode.as_mut().unwrap().optimized_build = true;
    c.bench_function("did_change_with_caching", |b| {
        let engines = Engines::default();
        b.iter(|| {
            for _ in 0..NUM_DID_CHANGE_ITERATIONS {
                let _ = black_box(
                    session::compile(&build_plan, &engines, None, lsp_mode.as_ref()).unwrap(),
                );
            }
        })
    });

    // Remove the temp dir after the benchmarks are done
    Runtime::new()
        .unwrap()
        .block_on(async { sync.remove_temp_dir() });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10)).sample_size(10);
    targets = benchmarks
}
