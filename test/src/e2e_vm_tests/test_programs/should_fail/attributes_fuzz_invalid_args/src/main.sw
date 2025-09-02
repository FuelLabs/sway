library;

// Invalid: fuzz argument without value assignment
#[test]
#[fuzz(invalid_arg)]
fn invalid_fuzz_with_unassigned_arg() {
}

// Invalid: case attribute requires arguments
#[test]
#[case()]
fn invalid_case_no_args() {
}

// Invalid: case attribute used without #[test]
#[case(some_value)]
fn invalid_case_without_test() {
}

// Invalid: fuzz attribute used without #[test]
#[fuzz(param_iterations = 10)]
fn invalid_fuzz_without_test() {
}