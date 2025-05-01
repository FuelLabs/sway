use codspeed_criterion_compat::{black_box, criterion_group, Criterion};
use forc_pkg::manifest::{GenericManifestFile, ManifestFile};
use lsp_types::Url;
use std::sync::Arc;
use parking_lot::RwLock;
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
        let engines_original = Arc::new(RwLock::new(Engines::default()));
        let engines_clone = engines_original.read().clone();
        let results = black_box(
            session::compile(&build_plan, &engines_clone, None, lsp_mode.as_ref()).unwrap(),
        );
        let session = Arc::new(session::Session::new());
        let member_path = sync.member_path(&uri).unwrap();

        b.iter(|| {
            let _ = black_box(
                session::traverse(
                    member_path.clone(),
                    results.clone(),
                    engines_original.clone(),
                    &engines_clone,
                    session.clone(),
                    &token_map,
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

    let examples_workspace_dir = super::sway_workspace_dir().join("examples");
    let member_manifests = ManifestFile::from_dir(examples_workspace_dir)
        .unwrap()
        .member_manifests()
        .unwrap();
    c.bench_function("open_all_example_workspace_members", |b| {
        b.iter(|| {
            let engines = Engines::default();
            for package_manifest in member_manifests.values() {
                let dir = Url::from_file_path(
                    package_manifest
                        .path()
                        .parent()
                        .unwrap()
                        .join("src/main.sw"),
                )
                .unwrap();
                let build_plan = session::build_plan(&dir).unwrap();
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
