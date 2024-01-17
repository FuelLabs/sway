predicate;

use std::auth::predicate_id;

fn main(predicate_address: Address) -> bool {
    // TODO: Converts an `Address` to a `PredicateId` until SDK supports the `PredicateId` type.
    let result_b256: b256 = predicate_address.into();
    PredicateId::from(result_b256) == predicate_id()
}

