pub mod compile;
pub mod requests;
pub mod token_map;

use lsp_types::Url;
use std::{path::PathBuf, sync::Arc};
use sway_core::{Engines, LspConfig};
use sway_lsp::{
    config::GarbageCollectionConfig,
    core::session::{self, Session},
    server_state::{CompilationContext, ServerState},
};

pub async fn compile_test_project() -> (Url, Arc<Session>, ServerState, Engines) {
    // Load the test project
    let uri = Url::from_file_path(benchmark_dir().join("src/main.sw")).unwrap();
    let state = ServerState::default();
    let engines_clone = state.engines.read().clone();
    let session = Arc::new(Session::new());
    let sync = state.get_or_init_global_sync_workspace(&uri).await.unwrap();
    let temp_uri = sync.workspace_to_temp_url(&uri).unwrap();

    state.documents.handle_open_file(&temp_uri).await;
    let ctx = CompilationContext {
        session: Some(session.clone()),
        sync: Some(sync.clone()),
        token_map: state.token_map.clone(),
        engines: state.engines.clone(),
        optimized_build: false,
        file_versions: Default::default(),
        uri: Some(uri.clone()),
        version: None,
        gc_options: GarbageCollectionConfig::default(),
    };
    let lsp_mode = Some(LspConfig {
        optimized_build: ctx.optimized_build,
        file_versions: ctx.file_versions.clone(),
    });

    // Compile the project
    session::parse_project(&temp_uri, &engines_clone, None, &ctx, lsp_mode.as_ref()).unwrap();
    (temp_uri, session, state, engines_clone)
}

pub fn sway_workspace_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

pub fn benchmark_dir() -> PathBuf {
    sway_workspace_dir().join("sway-lsp/tests/fixtures/benchmark")
}
