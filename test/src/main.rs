mod e2e_vm_tests;
mod ir_generation;

fn main() {
    let mut locked = false;
    let mut filter_regex = None;
    for arg in std::env::args().skip(1) {
        // Check for the `--locked` flag. Must precede the regex.
        // Intended for use in `CI` to ensure test lock files are up to date.
        if arg == "--locked" {
            locked = true;
            continue;
        }

        // Check for a regex, used to filter the set of tests.
        let regex = regex::Regex::new(&arg).unwrap_or_else(|_| {
            panic!(
                "Expected either `--locked` or a filter regex, found: {:?}.",
                arg
            )
        });
        filter_regex = Some(regex);
    }

    e2e_vm_tests::run(locked, filter_regex.as_ref());
    ir_generation::run(filter_regex.as_ref());
}
