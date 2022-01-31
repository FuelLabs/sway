use crate::priv_prelude::*;
use walkdir::WalkDir;

#[test]
fn test() {
    let program = crate::program();
    for entry_res in WalkDir::new("../test/src/e2e_vm_tests/test_programs") {
        let entry = entry_res.unwrap();
        if !entry.file_type().is_file() {
            continue;
        }
        match entry.path().extension() {
            Some(extension) if extension == "sw" => (),
            _ => continue,
        }
        let path = entry.path();
        println!("parsing {}", path.display());
        let bytes = std::fs::read(path).unwrap();
        let s = String::from_utf8(bytes).unwrap();
        let len = s.len();
        let span = Span::new(s.into(), 0, len);
        program.parse(&span).unwrap();
    }
}
