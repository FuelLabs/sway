predicate;

use std::auth::predicate_id;

fn main(predicate_address: Address) -> bool {
    let address = predicate_id();
    assert(predicate_address == address);

    true
}

