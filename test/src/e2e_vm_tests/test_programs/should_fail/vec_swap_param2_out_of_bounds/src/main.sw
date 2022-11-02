script;

use std::{assert::assert, vec::Vec};

fn main() {
    let mut vector = Vec::new();

    let number0 = 0u8;
    let number1 = 1u8;
    let number2 = 2u8;

    vector.push(number0);
    vector.push(number1);
    vector.push(number2);

    assert(vector.len() == 3);
    assert(vector.capacity() == 4);
    assert(vector.is_empty() == false);

    vector.swap(0, 3);
}
