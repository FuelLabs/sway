use forc_doc::{self, cli::Command, compile_html};
use std::path::Path;

#[test]
fn builds_lib_std_docs() {
    let path = Path::new("./../../sway-lib-std");
    let mut build_instructions = Command::default();
    build_instructions.manifest_path = Some(path.to_str().unwrap().to_string());
    println!("Building docs for {:?}", build_instructions.manifest_path);
    let res = compile_html(
        &build_instructions,
        sway_core::ExperimentalFlags {
            new_encoding: !build_instructions.no_encoding_v1,
        },
    );
    assert!(res.is_ok());
}
