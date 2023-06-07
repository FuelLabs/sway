script;

fn divide(numerator: u64, denominator: u64) -> Option<u64> {
    if denominator == 0 {
        None
    } else {
        Some(numerator / denominator)
    }
}

fn main() {
    let result = divide(6, 2);
    // Pattern match to retrieve the value
    match result {
        // The division was valid
        Some(x) => std::logging::log(x),
        // The division was invalid
        None => std::logging::log("Cannot divide by 0"),
    }
}
