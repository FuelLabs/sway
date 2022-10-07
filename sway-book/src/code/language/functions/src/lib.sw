library functions;

// ANCHOR: definition
fn my_function(my_parameter: u64, /* ... */) -> u64 {
    // function code
    42
}
// ANCHOR_END: definition

// ANCHOR: equals
fn equals(first_parameter: u64, second_parameter: u64) -> bool {
    first_parameter == second_parameter
}
// ANCHOR_END: equals

fn usage() {
    // ANCHOR: usage
    let result_one = equals(5, 5);  // evaluates to `true`
    let result_two = equals(5, 6);  // evaluates to `false`
    // ANCHOR_END: usage
}
