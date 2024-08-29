predicate;

use std::outputs::{Output, output_type};

fn main() -> bool {
    output_type(2).unwrap() == Output::ContractCreated
}
