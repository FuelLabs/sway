library lib_vec_test;

use core::ops::*;
use std::vec::*;

pub fn test_all<T>(
    value0: T,
    value1: T,
    value2: T,
    value3: T,
    value4: T,
    value5: T,
    value6: T,
    value7: T,
    value8: T,
) where T: Eq {
    without_capacity(value0, value1, value2, value3, value4, value5, value6, value7, value8);
    with_capacity(value0, value1, value2, value3, value4, value5, value6, value7, value8);
    swap(value0, value1, value2, value3);
}

pub fn without_capacity<T>(
    value0: T,
    value1: T,
    value2: T,
    value3: T,
    value4: T,
    value5: T,
    value6: T,
    value7: T,
    value8: T,
) where T: Eq {
    // create vector
    let mut vector: Vec<T> = Vec::new();

    assert_bounds(vector, 0, 0);

    // push 5 values
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);
    vector.push(value3);
    vector.push(value4);

    assert_bounds(vector, 5, 8);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);
    assert(vector.get(4).unwrap() == value4);

    // push 4 values
    vector.push(value5);
    vector.push(value6);
    vector.push(value7);
    vector.push(value8);

    assert_bounds(vector, 9, 16);
    assert(vector.get(5).unwrap() == value5);
    assert(vector.get(6).unwrap() == value6);
    assert(vector.get(7).unwrap() == value7);
    assert(vector.get(8).unwrap() == value8);

    // clear the vector
    vector.clear();

    assert_bounds(vector, 0, 16);
    assert(vector.get(0).is_none());

    // push 5 elements
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);
    vector.push(value3);
    vector.push(value4);

    assert_bounds(vector, 5, 16);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);
    assert(vector.get(4).unwrap() == value4);
    assert(vector.get(5).is_none());

    // remove first
    assert(vector.remove(0) == value0);

    assert_bounds(vector, 4, 16);
    assert(vector.get(0).unwrap() == value1);
    assert(vector.get(1).unwrap() == value2);
    assert(vector.get(2).unwrap() == value3);
    assert(vector.get(3).unwrap() == value4);

    // remove last
    assert(vector.remove(3) == value4);

    assert_bounds(vector, 3, 16);
    assert(vector.get(0).unwrap() == value1);
    assert(vector.get(1).unwrap() == value2);
    assert(vector.get(2).unwrap() == value3);

    // remove middle
    assert(vector.remove(1) == value2);

    assert_bounds(vector, 2, 16);
    assert(vector.get(0).unwrap() == value1);
    assert(vector.get(1).unwrap() == value3);

    // alloc new vec
    let mut vector = Vec::new();

    // insert into empty
    vector.insert(0, value2);

    assert_bounds(vector, 1, 1);
    assert(vector.get(0).unwrap() == value2);

    // insert at first
    vector.insert(0, value0);

    assert_bounds(vector, 2, 2);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value2);

    // insert at middle
    vector.insert(1, value1);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);

    // insert at last
    vector.insert(3, value3);

    assert_bounds(vector, 4, 4);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);

    // test pop
    vector.clear();
    vector.push(value0);
    vector.push(value1);

    assert_bounds(vector, 2, 4);

    // pop
    assert(vector.pop().unwrap() == value1);

    assert_bounds(vector, 1, 4);

    // pop
    assert(vector.pop().unwrap() == value0);

    assert_bounds(vector, 0, 4);

    // pop empty
    assert(vector.pop().is_none());

    assert_bounds(vector, 0, 4);

    // test for set
    vector.clear();
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);

    // set first
    vector.set(0, value3);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);

    // set middle
    vector.set(1, value4);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value4);
    assert(vector.get(2).unwrap() == value2);

    // set last
    vector.set(2, value5);

    assert_bounds(vector, 3, 4);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value4);
    assert(vector.get(2).unwrap() == value5);

    // 
}

pub fn with_capacity<T>(
    value0: T,
    value1: T,
    value2: T,
    value3: T,
    value4: T,
    value5: T,
    value6: T,
    value7: T,
    value8: T,
) where T: Eq {
    // create vector with capacity
    let mut vector = Vec::with_capacity(8);

    assert_bounds(vector, 0, 8);

    // push 5
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);
    vector.push(value3);
    vector.push(value4);

    assert_bounds(vector, 5, 8);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);
    assert(vector.get(4).unwrap() == value4);

    // push 4
    vector.push(value5);
    vector.push(value6);
    vector.push(value7);
    vector.push(value8);

    assert_bounds(vector, 9, 16);
    assert(vector.get(5).unwrap() == value5);
    assert(vector.get(6).unwrap() == value6);
    assert(vector.get(7).unwrap() == value7);
    assert(vector.get(8).unwrap() == value8);

    // clear
    vector.clear();

    assert_bounds(vector, 0, 16);
    assert(vector.get(0).is_none());

    // push 5
    vector.push(value0);
    vector.push(value1);
    vector.push(value2);
    vector.push(value3);
    vector.push(value4);

    assert_bounds(vector, 5, 16);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);
    assert(vector.get(3).unwrap() == value3);
    assert(vector.get(4).unwrap() == value4);
    assert(vector.get(5).is_none());
}

pub fn swap<T>(
    value0: T,
    value1: T,
    value2: T,
    value3: T,
) where T: Eq {
    let mut vector = Vec::new();

    vector.push(value0);
    vector.push(value1);
    vector.push(value2);

    assert_bounds(vector, 3, 4);

    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value2);

    vector.swap(0, 2);

    assert(vector.get(0).unwrap() == value2);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value0);
}

fn assert_bounds<T>(ref mut vector: Vec<T>, expected_len: u64, expected_cap: u64) {
    assert(vector.len() == expected_len);
    assert(vector.capacity() == expected_cap);
    assert(!vector.is_empty() || expected_len == 0);
}
