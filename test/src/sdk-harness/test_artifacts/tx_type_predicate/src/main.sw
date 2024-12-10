predicate;

use std::tx::{Transaction, tx_type};

fn main(expected_type: Transaction) -> bool {
    tx_type() == expected_type
}
