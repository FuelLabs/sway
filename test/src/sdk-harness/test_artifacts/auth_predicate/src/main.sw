predicate;

use std::auth::predicate_id;

fn main(predicate_address: Address) -> bool {
    predicate_address == predicate_id()
}

