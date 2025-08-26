library;

#[fuzz]
fn fuzz_simple() {
}

#[fuzz]
#[fuzz_param(name = "input1", iteration = 100)]
fn fuzz_with_single_param(input1: u64) {
}

#[fuzz]
#[fuzz_param(name = "input1", iteration = 50)]
#[fuzz_param(name = "input2", min_val = 0, max_val = 255)]
fn fuzz_with_multiple_params(input1: u64, input2: u8) {
}

#[fuzz]
#[fuzz_param(name = "x", min_val = 1, max_val = 100)]
#[fuzz_param(name = "y", iteration = 25)]
fn fuzz_with_mixed_params(x: u32, y: u64) {
}