pub mod compile;
pub mod requests;
pub mod token_map;

use lsp_types::Url;
use std::{path::PathBuf, sync::Arc};
use sway_lsp::core::session::{self, ParseResult, Session};

pub async fn compile_test_project() -> (Url, Arc<Session>) {
    let session = Session::new();
    let engines_clone = session.engines.read().clone();
    let lsp_mode = Some(sway_core::LspConfig {
        file_versions: Default::default(),
    });
    // Load the test project
    let uri = Url::from_file_path(benchmark_dir().join("src/main.sw")).unwrap();
    session.handle_open_file(&uri).await;
    // Compile the project and write the parse result to the session
    let mut parse_result = ParseResult::default();
    session::parse_project(&uri, &session.engines.read(), &engines_clone, None, &mut parse_result, lsp_mode).unwrap();
    session.write_parse_result(&mut parse_result);
    (uri, Arc::new(session))
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
