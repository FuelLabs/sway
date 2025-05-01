pub mod compile;
pub mod requests;
pub mod token_map;

use lsp_types::Url;
use parking_lot::RwLock;
use std::{path::PathBuf, sync::Arc};
use sway_core::Engines;
use sway_lsp::core::{
    document::Documents,
    session::{self, Session},
    token_map::TokenMap,
};

pub async fn compile_test_project() -> (
    Url,
    Arc<Session>,
    Documents,
    Arc<TokenMap>,
    Arc<RwLock<Engines>>,
) {
    let token_map = Arc::new(TokenMap::new());
    let engines = Arc::new(RwLock::new(Engines::default()));
    let engines_clone = engines.clone();
    let session = Arc::new(Session::new());
    let documents = Documents::new();
    let lsp_mode = Some(sway_core::LspConfig {
        optimized_build: false,
        file_versions: Default::default(),
    });
    // Load the test project
    let uri = Url::from_file_path(benchmark_dir().join("src/main.sw")).unwrap();
    documents.handle_open_file(&uri).await;
    // Compile the project
    session::parse_project(
        &uri,
        engines.clone(),
        &engines_clone.read(),
        None,
        lsp_mode,
        session.clone(),
        token_map.clone(),
    )
    .unwrap();
    (uri, session, documents, token_map, engines)
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
