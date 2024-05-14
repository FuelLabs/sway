predicate;

use std::inputs::input_predicate_data;

fn main() -> bool {
    let received: u32 = input_predicate_data::<u32>(0);
    let expected: u32 = 12345;

    received == expected
}
