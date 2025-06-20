use forc_doc::{self, cli::Command, generate_docs};
use std::path::Path;

#[test]
fn builds_lib_std_docs() {
    let path = Path::new("./../../sway-lib-std");
    let build_instructions = Command {
        path: Some(path.to_str().unwrap().to_string()),
        ..Default::default()
    };
    println!("Building docs for {:?}", build_instructions.path);
    let res = generate_docs(&build_instructions);
    assert!(res.is_ok());
}
