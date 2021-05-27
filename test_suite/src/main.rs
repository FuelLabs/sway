mod basic_compilation_tests;
mod e2e_vm_tests;

fn main() {
    basic_compilation_tests::run();
    e2e_vm_tests::run();
}
