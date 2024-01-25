pub mod compile;
pub mod requests;
pub mod token_map;

use lsp_types::Url;
use std::{path::PathBuf, sync::Arc};
use sway_lsp::core::session::{self, Session};

pub async fn compile_test_project() -> (Url, Arc<Session>) {
    let session = Arc::new(Session::new());
    // Load the test project
    let uri = Url::from_file_path(benchmark_dir().join("src/main.sw")).unwrap();
    session.handle_open_file(&uri).await;
    // Compile the project
    session::parse_project(&uri, &session.engines.read(), None, session.clone()).unwrap();
    (uri, session)
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
