library;

use std::bytes::Bytes;
use ::utils::setup;

#[test()]
fn bytes_splice() {
    let (mut bytes, a, b, c) = setup();
    bytes.push(11u8);
    bytes.push(13u8);
    // bytes = [5, 7, 9, 11, 13]

    // Remove [1..4), replace with nothing
    let spliced = bytes.splice(1, 4, Bytes::new());

    // bytes => [5, 13]
    assert(bytes.len() == 2);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == 13u8);

    // spliced => [7, 9, 11]
    assert(spliced.len() == 3);
    assert(spliced.get(0).unwrap() == b);
    assert(spliced.get(1).unwrap() == c);
    assert(spliced.get(2).unwrap() == 11u8);
}

#[test()]
fn bytes_splice_front() {
    let (mut bytes, a, b, c) = setup();
    // Remove [0..2) => [5, 7], replace with nothing
    let spliced = bytes.splice(0, 2, Bytes::new());

    // bytes => [9]
    assert(bytes.len() == 1);
    assert(bytes.get(0).unwrap() == c);

    // spliced => [5, 7]
    assert(spliced.len() == 2);
    assert(spliced.get(0).unwrap() == a);
    assert(spliced.get(1).unwrap() == b);
}

#[test()]
fn bytes_splice_end() {
    let (mut bytes, a, b, c) = setup();
    // Remove [1..3) => [7, 9], replace with nothing
    let spliced = bytes.splice(1, bytes.len(), Bytes::new());

    // bytes => [5]
    assert(bytes.len() == 1);
    assert(bytes.get(0).unwrap() == a);

    // spliced => [7, 9]
    assert(spliced.len() == 2);
    assert(spliced.get(0).unwrap() == b);
    assert(spliced.get(1).unwrap() == c);
}

#[test()]
fn bytes_splice_empty_range() {
    let (mut bytes, a, b, c) = setup();
    // Remove [1..1) => nothing, replace with nothing
    let spliced = bytes.splice(1, 1, Bytes::new());

    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
    assert(spliced.len() == 0);
}

#[test()]
fn bytes_splice_entire_range() {
    let (mut bytes, a, b, c) = setup();
    // Remove [0..3) => [5, 7, 9], replace with nothing
    let spliced = bytes.splice(0, bytes.len(), Bytes::new());

    assert(bytes.len() == 0);
    assert(bytes.is_empty());
    assert(spliced.len() == 3);
    assert(spliced.get(0).unwrap() == a);
    assert(spliced.get(1).unwrap() == b);
    assert(spliced.get(2).unwrap() == c);
}

#[test(should_revert)]
fn revert_bytes_splice_start_greater_than_end() {
    let (mut bytes, _, _, _) = setup();
    let _spliced = bytes.splice(2, 1, Bytes::new());
}

#[test(should_revert)]
fn revert_bytes_splice_end_out_of_bounds() {
    let (mut bytes, _, _, _) = setup();
    let _spliced = bytes.splice(0, bytes.len() + 1, Bytes::new());
}

/// Additional tests for replacing a spliced range with different Byte lengths.

#[test()]
fn bytes_splice_replace_smaller() {
    let (mut bytes, a, b, c) = setup();
    bytes.push(11u8);
    bytes.push(13u8);
    // bytes = [5, 7, 9, 11, 13]

    let mut replacement = Bytes::new();
    replacement.push(42u8);
    // Remove [1..4) => [7, 9, 11], replace with [42]
    let spliced = bytes.splice(1, 4, replacement);

    // bytes => [5, 42, 13]
    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == 42u8);
    assert(bytes.get(2).unwrap() == 13u8);

    // spliced => [7, 9, 11]
    assert(spliced.len() == 3);
    assert(spliced.get(0).unwrap() == b);
    assert(spliced.get(1).unwrap() == c);
    assert(spliced.get(2).unwrap() == 11u8);
}

#[test()]
fn bytes_splice_replace_larger() {
    let (mut bytes, a, b, c) = setup();
    // bytes = [5, 7, 9]
    let mut replacement = Bytes::new();
    replacement.push(42u8);
    replacement.push(50u8);
    replacement.push(60u8);
    // Remove [1..2) => [7], replace with [42, 50, 60]
    let spliced = bytes.splice(1, 2, replacement);

    // bytes => [5, 42, 50, 60, 9]
    assert(bytes.len() == 5);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == 42u8);
    assert(bytes.get(2).unwrap() == 50u8);
    assert(bytes.get(3).unwrap() == 60u8);
    assert(bytes.get(4).unwrap() == c);

    // spliced => [7]
    assert(spliced.len() == 1);
    assert(spliced.get(0).unwrap() == b);
}

#[test()]
fn bytes_splice_replace_same_length() {
    let (mut bytes, a, b, c) = setup();
    // bytes = [5, 7, 9]
    let mut replacement = Bytes::new();
    replacement.push(42u8);
    replacement.push(50u8);
    // Remove [1..3) => [7, 9], replace with [42, 50] (same length = 2)
    let spliced = bytes.splice(1, 3, replacement);

    // bytes => [5, 42, 50]
    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == 42u8);
    assert(bytes.get(2).unwrap() == 50u8);

    // spliced => [7, 9]
    assert(spliced.len() == 2);
    assert(spliced.get(0).unwrap() == b);
    assert(spliced.get(1).unwrap() == c);
}

#[test()]
fn bytes_splice_replace_empty_bytes() {
    let (mut bytes, a, b, c) = setup();
    // bytes = [5, 7, 9]
    let replacement = Bytes::new();
    // Remove [0..1) => [5], replace with []
    let spliced = bytes.splice(0, 1, replacement);

    // bytes => [7, 9]
    assert(bytes.len() == 2);
    assert(bytes.get(0).unwrap() == b);
    assert(bytes.get(1).unwrap() == c);

    // spliced => [5]
    assert(spliced.len() == 1);
    assert(spliced.get(0).unwrap() == a);
}
