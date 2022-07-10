predicate;

use std::tx::get_predicate_data;

struct Validation {
    has_account: bool,
    total_complete: u64
}

fn main() -> bool {
    let validation:Validation = get_predicate_data();
    validation.total_complete == 100 && validation.has_account
}
