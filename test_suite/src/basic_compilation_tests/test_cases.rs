pub fn run() {
    let project_names = vec!["script_1", "script_3"];
    assert!(project_names.into_iter().all(|name| {
        let result = crate::basic_compilation_tests::harness::should_compile(name);
        if !result {
            println!("Failure: {} should have compiled.", name);
            false
        } else {
            true
        }
    }));
}
