predicate;

use std::inputs::input_count;

fn main(expected_count: u16) -> bool {
    input_count() == expected_count
}
