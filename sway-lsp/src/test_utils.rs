#![allow(unused)]
use std::{env, path::PathBuf};
use tower_lsp::lsp_types::Url;

pub(crate) fn sway_workspace_dir() -> PathBuf {
    env::current_dir().unwrap().parent().unwrap().to_path_buf()
}

pub(crate) fn e2e_language_dir() -> PathBuf {
    PathBuf::from("test/src/e2e_vm_tests/test_programs/should_pass/language")
}

pub(crate) fn e2e_test_dir() -> PathBuf {
    sway_workspace_dir()
        .join(e2e_language_dir())
        .join("prelude_access2")
}

pub(crate) fn sway_example_dir() -> PathBuf {
    sway_workspace_dir().join("examples/storage_variables")
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
