library;

// Example 1: Simple fuzz test without specific parameters
#[test]
#[fuzz()]
fn fuzz_simple() {
}

// Example 2: Fuzz test with specific iterations for one parameter
#[test]
#[fuzz(input1_iterations = 100)]
fn fuzz_with_single_param(input1: u64) {
}

// Example 3: Mixed case and fuzz testing
#[test]
#[case(zero_input, small_input)]
#[fuzz(input1_iterations = 50, input2_min = 0, input2_max = 255)]
fn fuzz_with_mixed_fixtures(input1: u64, input2: u8) {
}

// Example 4: Multiple fuzz parameter configurations
#[test] 
#[fuzz(x_min = 1, x_max = 100, y_iterations = 25)]
fn fuzz_with_param_ranges(x: u32, y: u64) {
}