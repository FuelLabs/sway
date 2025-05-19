pub mod compile;
pub mod requests;
pub mod token_map;

use lsp_types::Url;
use std::{path::PathBuf, sync::Arc};
use sway_lsp::{
    core::session::{self, Session},
    server_state::ServerState,
};

pub async fn compile_test_project() -> (Url, Arc<Session>, ServerState) {
    // Load the test project
    let uri = Url::from_file_path(benchmark_dir().join("src/main.sw")).unwrap();
    let state = ServerState::default();
    let session = Arc::new(Session::new());
    let sync = state.get_or_init_global_sync_workspace(&uri).await.unwrap();
    let temp_uri = sync.workspace_to_temp_url(&uri).unwrap();

    let lsp_mode = Some(sway_core::LspConfig {
        optimized_build: false,
        file_versions: Default::default(),
    });

    state.documents.handle_open_file(&temp_uri).await;
    // Compile the project
    session::parse_project(
        &temp_uri,
        &session.engines.read(),
        None,
        lsp_mode,
        session.clone(),
        &sync,
    )
    .unwrap();
    (temp_uri, session, state)
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
