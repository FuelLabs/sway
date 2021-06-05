mod harness;

pub fn run() {
    let project_names = vec!["script_1", "script_2", "script_3", "script_4"];
    project_names.into_iter().for_each(|name| {
        crate::e2e_vm_tests::harness::runs_in_vm(name);
    });
}
