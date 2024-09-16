predicate;

use std::tx::{tx_type, Transaction};

fn main(expected_type: Transaction) -> bool {
    tx_type() == expected_type
}
