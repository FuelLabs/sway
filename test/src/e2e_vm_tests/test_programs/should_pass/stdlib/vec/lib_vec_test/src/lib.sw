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

    // create other vector
    let mut other_vector = Vec::new();
    other_vector.push(value0);
    other_vector.push(value1);
    other_vector.push(value2);

    assert_bounds(other_vector, 3, 4);
    assert(other_vector.get(0).unwrap() == value0);
    assert(other_vector.get(1).unwrap() == value1);
    assert(other_vector.get(2).unwrap() == value2);

    // append other vector to vector
    vector.append(other_vector);

    assert_bounds(vector, 6, 6);
    assert_bounds(other_vector, 0, 4);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value4);
    assert(vector.get(2).unwrap() == value5);
    assert(vector.get(3).unwrap() == value0);
    assert(vector.get(4).unwrap() == value1);
    assert(vector.get(5).unwrap() == value2);
    assert(other_vector.get(0).is_none());

    // split off vector
    let mut other_vector = vector.split_off(3);

    assert_bounds(vector, 3, 6);
    assert_bounds(other_vector, 3, 3);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value4);
    assert(vector.get(2).unwrap() == value5);
    assert(other_vector.get(3).unwrap() == value0);
    assert(other_vector.get(4).unwrap() == value1);
    assert(other_vector.get(5).unwrap() == value2);

    // append other vector to vector
    vector.append(other_vector);

    assert_bounds(vector, 6, 6);
    assert_bounds(other_vector, 0, 3);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value4);
    assert(vector.get(2).unwrap() == value5);
    assert(vector.get(3).unwrap() == value0);
    assert(vector.get(4).unwrap() == value1);
    assert(vector.get(5).unwrap() == value2);
    assert(other_vector.get(0).is_none());

    // split at index
    let (mut vector, mut other_vector) = vector.split_at(3);

    assert_bounds(vector, 3, 3);
    assert_bounds(other_vector, 3, 3);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value4);
    assert(vector.get(2).unwrap() == value5);
    assert(other_vector.get(3).unwrap() == value0);
    assert(other_vector.get(4).unwrap() == value1);
    assert(other_vector.get(5).unwrap() == value2);

    // append other vector to vector
    vector.append(other_vector);

    assert_bounds(vector, 6, 6);
    assert_bounds(other_vector, 0, 3);
    assert(vector.get(0).unwrap() == value3);
    assert(vector.get(1).unwrap() == value4);
    assert(vector.get(2).unwrap() == value5);
    assert(vector.get(3).unwrap() == value0);
    assert(vector.get(4).unwrap() == value1);
    assert(vector.get(5).unwrap() == value2);
    assert(other_vector.get(0).is_none());

    // check first and last
    assert(vector.first().unwrap() == value3);
    assert(vector.last().unwrap() == value2);
    assert(other_vector.first().is_none());
    assert(other_vector.last().is_none());

    // reverse vector
    vector.reverse();

    assert_bounds(vector, 6, 6);
    assert(vector.get(0).unwrap() == value2);
    assert(vector.get(1).unwrap() == value1);
    assert(vector.get(2).unwrap() == value0);
    assert(vector.get(3).unwrap() == value5);
    assert(vector.get(4).unwrap() == value4);
    assert(vector.get(5).unwrap() == value3);

    // fill vector with first value
    vector.fill(value0);

    assert_bounds(vector, 6, 6);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value0);
    assert(vector.get(2).unwrap() == value0);
    assert(vector.get(3).unwrap() == value0);
    assert(vector.get(4).unwrap() == value0);
    assert(vector.get(5).unwrap() == value0);

    // resize vector
    vector.resize(8, value1);

    assert_bounds(vector, 8, 8);
    assert(vector.get(0).unwrap() == value0);
    assert(vector.get(1).unwrap() == value0);
    assert(vector.get(2).unwrap() == value0);
    assert(vector.get(3).unwrap() == value0);
    assert(vector.get(4).unwrap() == value0);
    assert(vector.get(5).unwrap() == value0);
    assert(vector.get(6).unwrap() == value1);
    assert(vector.get(7).unwrap() == value1);

    // test contains
    assert(Vec::contains(vector, value0));
    assert(Vec::contains(vector, value1));
    assert(!Vec::contains(vector, value2));
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

pub fn sort<T>(
    value0: T,
    value1: T,
) where T: Ord + Eq {
    // create new vector
    let mut vector = Vec::with_capacity(2);

    // push 2
    vector.push(value0);
    vector.push(value1);

    // reverse if sorted
    if vector.is_sorted() {
        vector.reverse();
    }

    // sort
    Vec::sort(vector);

    // assert sorted
    assert(vector.is_sorted());

    // push 1
    vector.push(value3);

    // swap if sorted
    if vector.is_sorted() {
        vector.swap(0, 2);
    }

    Vec::sort(vector);

    // assert sorted
    assert(vector.is_sorted());
}

fn assert_bounds<T>(ref mut vector: Vec<T>, expected_len: u64, expected_cap: u64) {
    assert(vector.len() == expected_len);
    assert(vector.capacity() == expected_cap);
    assert(!vector.is_empty() || expected_len == 0);
}

// TODO: implement this as `Vec::is_sorted`.
fn is_sorted<T>(ref mut vector: Vec<T>) -> bool where T: Ord {
    let len_sub_one = vector.len() - 1;
    let mut i = 0;
    while i < len_sub_one {
        if vector.get(i).unwrap() > vector.get(i + 1).unwrap() {
            return false;
        }
        i += 2;
    }
    true
}

fn shuffle(ref mut vector: Vec<T>, seed: Option<u64>) -> Vec<T> {
    let hash = std::hash::sha256(seed.unwrap_or(1));
    let len = vector.len();
    let mut i = 0;
    while i < 31 {
        let src_index = __addr_of(hash).add::<b256>(i).read_byte() % len;
        let mut dest_index = _addr_of(hash).add::<b256>(i + 1).read_byte() % len;
        if src_index == dest_index {
            dest_index = (dest_index + 1) % len;
        }
        vector.swap(src_index, dest_index);
        i += 2;
    }
}
