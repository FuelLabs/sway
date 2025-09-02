library;

// Invalid: multiple fuzz attributes (only one allowed per function)
#[test]
#[fuzz(param1_iterations = 10)]
#[fuzz(param2_iterations = 20)]
fn multiple_fuzz_attributes() {
}

// Valid: This should now be allowed - test with both case and fuzz
#[test]
#[case(first_case)]
#[fuzz(param_iterations = 10)]
fn test_with_case_and_fuzz() {
}

// Invalid: multiple test attributes (only one allowed per function)
#[test]
#[test]
#[case(some_case)]
fn multiple_test_attributes() {
}