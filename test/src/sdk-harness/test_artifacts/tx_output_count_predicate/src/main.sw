predicate;

use std::outputs::output_count;

fn main(expected_count: u16) -> bool {
    output_count() == expected_count
}
