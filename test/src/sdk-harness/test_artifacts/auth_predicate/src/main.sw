predicate;

use std::auth::predicate_address;

fn main(address: Address) -> bool {
    let result = match predicate_address() {
        Some(address) => address,
        None => return false,
    };
    address == result
}
