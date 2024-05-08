use forc_doc::{self, cli::Command, compile_html, get_doc_dir};
use std::path::{Path, PathBuf};

#[test]
fn builds_lib_std_docs() {
    let path = Path::new("./../../sway-lib-std");
    let build_instructions = Command {
        manifest_path: Some(path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    println!("Building docs for {:?}", build_instructions.manifest_path);
    let res = compile_html(
        &build_instructions,
        &get_doc_dir,
        sway_core::ExperimentalFlags {
            new_encoding: !build_instructions.no_encoding_v1,
        },
    );
    assert!(res.is_ok());
}

#[test]
fn function_impls() {
    let path = test_fixtures_dir().join("function_impls");
    let build_instructions = Command {
        manifest_path: Some(path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    println!("Building docs for {:?}", build_instructions.manifest_path);
    let res = compile_html(
        &build_instructions,
        &get_doc_dir,
        sway_core::ExperimentalFlags {
            new_encoding: !build_instructions.no_encoding_v1,
        },
    );
    assert!(res.is_ok());
}

pub fn sway_workspace_dir() -> PathBuf {
    std::env::current_dir().unwrap().parent().unwrap().to_path_buf()
}

pub fn test_fixtures_dir() -> PathBuf {
    sway_workspace_dir().join("forc-doc/tests/fixtures")
}