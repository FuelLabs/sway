library;

// ANCHOR: main
fn main() -> bool {
    true
}
// ANCHOR_END: main
// ANCHOR: return_data
fn return_data(parameter_one: u64, parameter_two: bool) -> (bool, u64) {
    if parameter_two {
        (!parameter_two, parameter_one + 42)
    } else {
        (parameter_two, 42)
    }
}
// ANCHOR_END: return_data
