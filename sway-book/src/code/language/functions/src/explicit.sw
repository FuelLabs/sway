library explicit;

// ANCHOR: main
fn main() -> bool {
    return true;
}
// ANCHOR_END: main
// ANCHOR: return_data
fn return_data(
    parameter_one: u64,
    parameter_two: b256,
    parameter_three: bool,
) -> (b256, bool, u64) {
    // if parameter_three is true
    if parameter_three {
        return (
            parameter_two,
            parameter_three,
            parameter_one * 2,
        );
    }

    // some code here
    return (
        parameter_two,
        false,
        42,
    );
}
// ANCHOR_END: return_data
