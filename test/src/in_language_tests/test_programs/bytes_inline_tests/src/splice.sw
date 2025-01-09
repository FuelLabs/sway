library;

use std::bytes::Bytes;
use ::utils::setup;

#[test()]
fn bytes_splice_regular() {
    // Basic splice operation in the middle
    let (mut bytes, a, b, c) = setup();
    // bytes = [a=5, b=7, c=9]
    // Add two more elements for better illustration
    bytes.push(11u8); // 4th
    bytes.push(13u8); // 5th
    // bytes = [5,7,9,11,13]
    assert(bytes.len() == 5);

    // Splice out the range [1, 4)
    // That should return items at indices 1..3: [7, 9, 11]
    let spliced = bytes.splice(1, 4);

    // The original bytes should now have the front element [5]
    // plus the last element [13] left => [5, 13]
    assert(bytes.len() == 2);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == 13u8);

    // The spliced bytes should contain [b=7, c=9, 11]
    assert(spliced.len() == 3);
    assert(spliced.get(0).unwrap() == b);
    assert(spliced.get(1).unwrap() == c);
    assert(spliced.get(2).unwrap() == 11u8);
}

#[test()]
fn bytes_splice_front() {
    // Splice from front
    let (mut bytes, a, b, c) = setup();
    // bytes = [5, 7, 9]
    assert(bytes.len() == 3);

    // Splice [0, 2): should remove elements at indices 0..1 => [5, 7]
    let spliced = bytes.splice(0, 2);

    // The original bytes should have only [9] left
    assert(bytes.len() == 1);
    assert(bytes.get(0).unwrap() == c);
    assert(bytes.get(1).is_none());

    // The spliced bytes should be [5, 7]
    assert(spliced.len() == 2);
    assert(spliced.get(0).unwrap() == a);
    assert(spliced.get(1).unwrap() == b);
}

#[test()]
fn bytes_splice_end() {
    // Splice until the end
    let (mut bytes, a, b, c) = setup();
    // bytes = [5, 7, 9]
    assert(bytes.len() == 3);

    // Splice out the range [1, 3) => [7, 9]
    let spliced = bytes.splice(1, bytes.len());

    // The original bytes should have only [5]
    assert(bytes.len() == 1);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).is_none());

    // The spliced bytes should be [7, 9]
    assert(spliced.len() == 2);
    assert(spliced.get(0).unwrap() == b);
    assert(spliced.get(1).unwrap() == c);
}

#[test()]
fn bytes_splice_empty_range() {
    let (mut bytes, a, b, c) = setup();
    // bytes = [5, 7, 9]

    // Splice a zero-length range [1, 1): returns nothing
    let spliced = bytes.splice(1, 1);

    // Original bytes are unchanged
    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);

    // Spliced bytes is empty
    assert(spliced.len() == 0);
}

#[test()]
fn bytes_splice_entire_range() {
    // Splice everything out
    let (mut bytes, a, b, c) = setup();
    // bytes = [5, 7, 9]
    assert(bytes.len() == 3);

    // Splice out everything: [0, 3)
    let spliced = bytes.splice(0, bytes.len());

    // Original bytes should now be empty
    assert(bytes.len() == 0);
    assert(bytes.is_empty());

    // Spliced has [5, 7, 9]
    assert(spliced.len() == 3);
    assert(spliced.get(0).unwrap() == a);
    assert(spliced.get(1).unwrap() == b);
    assert(spliced.get(2).unwrap() == c);
}

#[test(should_revert)]
fn revert_bytes_splice_start_greater_than_end() {
    let (mut bytes, _a, _b, _c) = setup();
    // Attempt to splice a range where start > end
    let _spliced = bytes.splice(2, 1);
}

#[test(should_revert)]
fn revert_bytes_splice_end_out_of_bounds() {
    let (mut bytes, _a, _b, _c) = setup();
    // Attempt to splice out-of-bounds: end = len + 1
    let _spliced = bytes.splice(0, bytes.len() + 1);
}
