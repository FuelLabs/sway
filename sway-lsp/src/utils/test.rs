#![allow(unused)]
use assert_json_diff::assert_json_include;
use futures::StreamExt;
use serde_json::Value;
use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::task::JoinHandle;
use tower_lsp::{lsp_types::Url, ClientSocket};

pub(crate) fn sway_workspace_dir() -> PathBuf {
    env::current_dir().unwrap().parent().unwrap().to_path_buf()
}

pub(crate) fn e2e_language_dir() -> PathBuf {
    PathBuf::from("test/src/e2e_vm_tests/test_programs/should_pass/language")
}

pub(crate) fn e2e_unit_dir() -> PathBuf {
    PathBuf::from("test/src/e2e_vm_tests/test_programs/should_pass/unit_tests")
}

pub(crate) fn e2e_test_dir() -> PathBuf {
    sway_workspace_dir()
        .join(e2e_language_dir())
        .join("struct_field_access")
}

pub(crate) fn runnables_test_dir() -> PathBuf {
    sway_workspace_dir()
        .join(e2e_unit_dir())
        .join("script_multi_test")
}

pub(crate) fn test_fixtures_dir() -> PathBuf {
    sway_workspace_dir().join("sway-lsp/test/fixtures")
}

pub(crate) fn doc_comments_dir() -> PathBuf {
    sway_workspace_dir()
        .join(e2e_language_dir())
        .join("doc_comments")
}

pub(crate) fn get_absolute_path(path: &str) -> String {
    sway_workspace_dir().join(path).to_str().unwrap().into()
}

pub(crate) fn get_url(absolute_path: &str) -> Url {
    Url::parse(&format!("file://{}", &absolute_path)).expect("expected URL")
}

pub(crate) fn get_fixture(path: PathBuf) -> Value {
    let text = std::fs::read_to_string(path).expect("Failed to read file");
    serde_json::from_str::<Value>(&text).expect("Failed to parse JSON")
}

pub(crate) fn sway_example_dir() -> PathBuf {
    sway_workspace_dir().join("examples/storage_variables")
}

// Check if the given directory contains `Forc.toml` at its root.
pub(crate) fn dir_contains_forc_manifest(path: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if entry.path().file_name().and_then(|s| s.to_str()) == Some("Forc.toml") {
                return true;
            }
        }
    }
    false
}

pub(crate) async fn assert_server_requests(
    socket: ClientSocket,
    expected_requests: Vec<Value>,
    timeout: Option<Duration>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let request_stream = socket.take(expected_requests.len()).collect::<Vec<_>>();
        let requests =
            tokio::time::timeout(timeout.unwrap_or(Duration::from_secs(5)), request_stream)
                .await
                .expect("Timed out waiting for requests from server");

        assert_eq!(requests.len(), expected_requests.len());
        for (actual, expected) in requests.iter().zip(expected_requests.iter()) {
            assert_eq!(expected["method"], actual.method());

            // Assert that all other expected fields are present without requiring
            // all actual fields to be present. Specifically we need this for `uri`,
            // which can't be hardcoded in the test.
            assert_json_include!(
                expected: expected,
                actual: serde_json::to_value(actual.clone()).unwrap()
            );
        }
    })
}
