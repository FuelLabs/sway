mod harness;

pub fn run() {
    let project_names = vec!["script_1", "script_2", "script_3"];
    assert!(project_names.into_iter().all(|name| {
        let result = crate::e2e_vm_tests::harness::runs_in_vm(name);
        if !result {
            println!("E2E Failure: {} should have run in the VM.", name);
            false
        } else {
            true
        }
    }));
}
