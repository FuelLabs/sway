#![cfg(test)]
use crate::{cli::Command, compile_html};
use expect_test::Expect;
use std::path::PathBuf;

mod impl_trait;

pub(crate) fn check(command: Command, path_to_file: PathBuf, expect: &Expect) {
    let (doc_path, _) = compile_html(&command, &get_doc_dir).unwrap();
    let actual = std::fs::read_to_string(doc_path.join(path_to_file)).unwrap();
    expect.assert_eq(&actual)
}

fn get_doc_dir(build_instructions: &Command) -> String {
    build_instructions.doc_path.to_owned().unwrap()
}
