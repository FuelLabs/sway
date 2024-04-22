predicate;

use std::auth::predicate_address;

fn main(address: Address) -> bool {
    address == predicate_address()
}
