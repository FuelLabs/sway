use forc_doc::{self, cli::Command, compile_html, get_doc_dir};
use std::path::Path;

#[test]
fn builds_lib_std_docs() {
    let path = Path::new("./../../sway-lib-std");
    let build_instructions = Command {
        manifest_path: Some(path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    println!("Building docs for {:?}", build_instructions.manifest_path);
    let res = compile_html(&build_instructions, &get_doc_dir);
    assert!(res.is_ok());
}
