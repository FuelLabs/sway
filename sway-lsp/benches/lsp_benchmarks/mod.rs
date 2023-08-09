pub mod requests;
pub mod token_map;

use lsp_types::Url;
use std::{path::PathBuf, sync::Arc};
use sway_lsp::core::session::{self, Session};

pub fn compile_test_project() -> (Url, Arc<Session>) {
    let session = Session::new();
    // Load the test project
    let uri = Url::from_file_path(PathBuf::from("/Users/josh/Documents/rust/fuel/lsp-test-projects/lsp_benchmarking/sway_project/src/main.sw")).unwrap();
    session.handle_open_file(&uri);
    // Compile the project and write the parse result to the session
    let parse_result = session::parse_project(&uri).unwrap();
    session.write_parse_result(parse_result);
    (uri, Arc::new(session))
}
