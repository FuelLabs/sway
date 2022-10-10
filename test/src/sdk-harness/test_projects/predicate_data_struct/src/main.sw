predicate;

use std::inputs::input_predicate_data;

struct Validation {
    has_account: bool,
    total_complete: u64,
}

fn main() -> bool {
    let validation: Validation = input_predicate_data(0);
    validation.total_complete == 100 && validation.has_account
}
