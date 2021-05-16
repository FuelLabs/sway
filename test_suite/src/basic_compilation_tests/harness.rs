use forc;

/// Returns `true` if a file compiled without any errors or warnings,
/// and `false` if it did not.
pub(crate) fn should_compile(file_name: &str) -> bool {
    println!("Compiling {}", file_name);
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let res = forc::build(Some(format!(
        "{}/src/basic_compilation_tests/test_programs/{}",
        manifest_dir, file_name
    )));
    match res {
        Ok(_) => true,
        Err(_) => {
            println!("Project \"{}\" failed to compile. ", file_name);
            return false;
        }
    }
}
