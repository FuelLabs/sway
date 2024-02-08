#![cfg(test)]
use crate::cli::Command;
use expect_test::Expect;
use std::path::{Path, PathBuf};

mod impl_trait;

pub(crate) fn check_file(doc_path: &Path, path_to_file: &PathBuf, expect: &Expect) {
    let path = doc_path.join(path_to_file);
    let actual = std::fs::read_to_string(path.clone())
        .unwrap_or_else(|_| panic!("failed to read file: {:?}", path));
    expect.assert_eq(&actual)
}

pub(crate) fn get_doc_dir(build_instructions: &Command) -> String {
    build_instructions.doc_path.to_owned().unwrap()
}
