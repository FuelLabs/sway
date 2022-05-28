script;

use std::assert::assert;
use std::option::Option;
use std::revert::revert;
use std::vec::Vec;

fn main() -> bool {
    test_vector_new();
    true
}

fn test_vector_new() {
    let v: Vec<u64> = ~Vec::new::<u64>();

    v.push(42);

    assert(v.len() == 1);
    let val = v.get(0);
    if let Option::Some(inner_value) = val {
        assert(42 == inner_value);
    } else {
        revert(0);
    }
}
