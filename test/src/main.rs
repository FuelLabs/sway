mod e2e_vm_tests;

fn main() {
    let filter_regex = std::env::args().nth(1).map(|filter_str| {
        regex::Regex::new(&filter_str)
            .unwrap_or_else(|_| panic!("Invalid filter regex: '{}'.", filter_str))
    });

    e2e_vm_tests::run(filter_regex);
}
