library;

use std::bytes::Bytes;

fn setup() -> (Bytes, u8, u8, u8) {
    let mut bytes = Bytes::new();
    let a = 5u8;
    let b = 7u8;
    let c = 9u8;
    bytes.push(a);
    bytes.push(b);
    bytes.push(c);
    (bytes, a, b, c)
}

#[test()]
fn bytes_new() {
    let bytes = Bytes::new();
    assert(bytes.len() == 0);
    assert(bytes.capacity() == 0);
}

#[test]
fn bytes_with_capacity() {
    let bytes_1 = Bytes::with_capacity(0);
    assert(bytes_1.capacity() == 0);

    let bytes_2 = Bytes::with_capacity(1);
    assert(bytes_2.capacity() == 1);

    // 2^6
    let bytes_3 = Bytes::with_capacity(64);
    assert(bytes_3.capacity() == 64);

    // 2^11
    let bytes_4 = Bytes::with_capacity(2048);
    assert(bytes_4.capacity() == 2048);

    // 2^16
    let bytes_5 = Bytes::with_capacity(65536);
    assert(bytes_5.capacity() == 65536);
}

#[test()]
fn bytes_push() {
    let mut bytes = Bytes::new();

    assert(bytes.len() == 0);
    assert(bytes.capacity() == 0);

    bytes.push(1u8);
    assert(bytes.len() == 1);
    assert(bytes.capacity() == 1);

    bytes.push(2u8);
    assert(bytes.len() == 2);
    assert(bytes.capacity() == 2);

    // Capacity doubles
    bytes.push(3u8);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);

    bytes.push(4u8);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);

    // Capacity doubles
    bytes.push(5u8);
    assert(bytes.len() == 5);
    assert(bytes.capacity() == 8);

    bytes.push(6u8);
    assert(bytes.len() == 6);
    assert(bytes.capacity() == 8);

    bytes.push(7u8);
    assert(bytes.len() == 7);
    assert(bytes.capacity() == 8);

    bytes.push(8u8);
    assert(bytes.len() == 8);
    assert(bytes.capacity() == 8);

    // Capacity doubles
    bytes.push(9u8);
    assert(bytes.len() == 9);
    assert(bytes.capacity() == 16);
}

#[test()]
fn bytes_pop() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);

    bytes.push(42u8);
    bytes.push(11u8);
    bytes.push(69u8);
    bytes.push(100u8);
    bytes.push(200u8);
    bytes.push(255u8);
    bytes.push(180u8);
    bytes.push(17u8);
    bytes.push(19u8);
    assert(bytes.len() == 12);
    assert(bytes.capacity() == 16);

    let first = bytes.pop();
    assert(first.unwrap() == 19u8);
    assert(bytes.len() == 11);
    assert(bytes.capacity() == 16);

    let second = bytes.pop();
    assert(second.unwrap() == 17u8);
    assert(bytes.len() == 10);
    assert(bytes.capacity() == 16);

    let third = bytes.pop();
    assert(third.unwrap() == 180u8);
    assert(bytes.len() == 9);
    let _ = bytes.pop();
    let _ = bytes.pop();
    let _ = bytes.pop();
    let _ = bytes.pop();
    let _ = bytes.pop();
    let _ = bytes.pop();
    assert(bytes.len() == 3);
    assert(bytes.pop().unwrap() == c);
    assert(bytes.pop().unwrap() == b);
    assert(bytes.pop().unwrap() == a);

    // Can pop all
    assert(bytes.len() == 0);
    assert(bytes.capacity() == 16);
    assert(bytes.pop().is_none());
}

#[test()]
fn bytes_get() {
    let (bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
    // get is non-modifying
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
    assert(bytes.len() == 3);

    // None if out of bounds
    assert(bytes.get(bytes.len()).is_none());
}

#[test()]
fn bytes_set() {
    let (mut bytes, a, _b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    let d = 11u8;

    // Sets in the middle
    bytes.set(1, d);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == d);
    assert(bytes.get(2).unwrap() == c);
}

#[test()]
fn bytes_set_twice() {
    let (mut bytes, a, _b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    let d = 11u8;
    let e = 13u8;

    // Sets in the middle
    bytes.set(1, d);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == d);
    assert(bytes.get(2).unwrap() == c);

    // Twice
    bytes.set(1, e);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == e);
    assert(bytes.get(2).unwrap() == c);
}

#[test()]
fn bytes_set_front() {
    let (mut bytes, _a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    let d = 11u8;

    // Sets at the front
    bytes.set(0, d);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == d);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
}

#[test()]
fn bytes_set_back() {
    let (mut bytes, a, b, _c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    let d = 11u8;

    // Sets at the back
    bytes.set(bytes.len() - 1, d);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == d);
}

#[test(should_revert)]
fn revert_bytes_set_out_of_bounds() {
    let (mut bytes, _a, _b, _c) = setup();

    bytes.set(bytes.len(), 11u8);
}

#[test()]
fn bytes_insert() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);

    let d = 11u8;

    // Inserts in the middle
    bytes.insert(1, d);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == d);
    assert(bytes.get(2).unwrap() == b);
    assert(bytes.get(3).unwrap() == c);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);
}

#[test()]
fn bytes_insert_twice() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);

    let d = 11u8;
    let e = 13u8;

    // Inserts in the middle
    bytes.insert(1, d);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == d);
    assert(bytes.get(2).unwrap() == b);
    assert(bytes.get(3).unwrap() == c);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);

    // Twice
    bytes.insert(1, e);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == e);
    assert(bytes.get(2).unwrap() == d);
    assert(bytes.get(3).unwrap() == b);
    assert(bytes.get(4).unwrap() == c);
    assert(bytes.len() == 5);
    assert(bytes.capacity() == 8);
}

#[test()]
fn bytes_insert_front() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);

    let d = 11u8;

    // Inserts at the front
    bytes.insert(0, d);
    assert(bytes.get(0).unwrap() == d);
    assert(bytes.get(1).unwrap() == a);
    assert(bytes.get(2).unwrap() == b);
    assert(bytes.get(3).unwrap() == c);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);
}

#[test()]
fn bytes_insert_before_back() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);

    let d = 11u8;

    // Inserts right before the back
    bytes.insert(bytes.len() - 1, d);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == d);
    assert(bytes.get(3).unwrap() == c);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);
}

#[test()]
fn bytes_insert_back() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);

    let d = 11u8;

    // Inserts at the back
    bytes.insert(bytes.len(), d);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
    assert(bytes.get(3).unwrap() == d);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);
}

#[test(should_revert)]
fn revert_bytes_insert_out_of_bounds() {
    let (mut bytes, a, _b, _c) = setup();

    bytes.insert(bytes.len() + 1, a);
}

#[test()]
fn bytes_remove() {
    let (mut bytes, a, b, c) = setup();
    let d = 7u8;
    bytes.push(d);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);

    // Remove in the middle
    let item1 = bytes.remove(1);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    assert(item1 == b);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == c);
    assert(bytes.get(2).unwrap() == d);
    assert(bytes.get(3).is_none());
}

#[test()]
fn bytes_remove_front() {
    let (mut bytes, a, b, c) = setup();
    // Remove at the start
    let item = bytes.remove(0);
    assert(bytes.len() == 2);
    assert(bytes.capacity() == 4);
    assert(item == a);
    assert(bytes.get(0).unwrap() == b);
    assert(bytes.get(1).unwrap() == c);
    assert(bytes.get(2).is_none());
}

#[test()]
fn bytes_remove_end() {
    let (mut bytes, a, b, c) = setup();
    // Remove at the end
    let item = bytes.remove(bytes.len() - 1);
    assert(bytes.len() == 2);
    assert(bytes.capacity() == 4);
    assert(item == c);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).is_none());
}

#[test()]
fn bytes_remove_all() {
    let (mut bytes, a, b, c) = setup();
    // Remove all
    let item1 = bytes.remove(0);
    let item2 = bytes.remove(0);
    let item3 = bytes.remove(0);
    assert(bytes.len() == 0);
    assert(bytes.capacity() == 4);
    assert(item1 == a);
    assert(item2 == b);
    assert(item3 == c);
    assert(bytes.get(0).is_none());
}

#[test(should_revert)]
fn revert_bytes_remove_out_of_bounds() {
    let (mut bytes, _a, _b, _c) = setup();

    let _result = bytes.remove(bytes.len());
}

#[test()]
fn bytes_swap() {
    let (mut bytes, a, b, c) = setup();
    let d = 5u8;
    bytes.push(d);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);

    // Swaps Middle
    bytes.swap(1, 2);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == c);
    assert(bytes.get(2).unwrap() == b);
    assert(bytes.get(3).unwrap() == d);
}

#[test()]
fn bytes_swap_twice() {
    let (mut bytes, a, b, c) = setup();
    let d = 5u8;
    bytes.push(d);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);

    // Swaps Middle
    bytes.swap(1, 2);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == c);
    assert(bytes.get(2).unwrap() == b);
    assert(bytes.get(3).unwrap() == d);

    bytes.swap(1, 2);
    assert(bytes.len() == 4);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
    assert(bytes.get(3).unwrap() == d);
}

#[test()]
fn bytes_swap_front() {
    let (mut bytes, a, b, c) = setup();

    // Swaps Front
    bytes.swap(0, 1);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == b);
    assert(bytes.get(1).unwrap() == a);
    assert(bytes.get(2).unwrap() == c);
}

#[test()]
fn bytes_swap_end() {
    let (mut bytes, a, b, c) = setup();

    // Swaps back
    bytes.swap(2, 1);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == c);
    assert(bytes.get(2).unwrap() == b);
}

#[test()]
fn bytes_swap_front_with_end() {
    let (mut bytes, a, b, c) = setup();

    // Swaps front with back
    bytes.swap(0, 2);
    assert(bytes.len() == 3);
    assert(bytes.capacity() == 4);
    assert(bytes.get(0).unwrap() == c);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == a);
}

#[test(should_revert)]
fn revert_bytes_swap_element_1_out_of_bounds() {
    let (mut bytes, _a, _b, _c) = setup();

    bytes.swap(bytes.len(), 0);
}

#[test(should_revert)]
fn revert_bytes_swap_element_2_out_of_bounds() {
    let (mut bytes, _a, _b, _c) = setup();

    bytes.swap(0, bytes.len());
}

#[test()]
fn bytes_capacity() {
    let mut bytes = Bytes::new();
    assert(bytes.capacity() == 0);

    bytes.push(5u8);
    assert(bytes.capacity() == 1);
    bytes.push(7u8);
    assert(bytes.capacity() == 2);
    bytes.push(9u8);
    assert(bytes.capacity() == 4);
    bytes.push(11u8);
    assert(bytes.capacity() == 4);
    bytes.push(3u8);
    assert(bytes.capacity() == 8);
}

#[test()]
fn bytes_len() {
    let (mut bytes, _, _, _) = setup();
    assert(bytes.len() == 3);

    bytes.push(5u8);
    assert(bytes.len() == 4);
    bytes.push(6u8);
    assert(bytes.len() == 5);
    bytes.push(7u8);
    assert(bytes.len() == 6);
    bytes.push(8u8);
    assert(bytes.len() == 7);
}

#[test()]
fn bytes_clear() {
    let (mut bytes, _, _, _) = setup();
    assert(bytes.len() == 3);

    bytes.clear();
    assert(bytes.len() == 0);
    assert(bytes.capacity() == 0);
}

#[test()]
fn bytes_clear_twice() {
    let (mut bytes, _, _, _) = setup();

    bytes.clear();
    assert(bytes.len() == 0);
    assert(bytes.capacity() == 0);

    // Can clean twice
    bytes.push(1u8);
    bytes.clear();
    assert(bytes.len() == 0);
    assert(bytes.capacity() == 0);
}

#[test()]
fn bytes_clear_empty_bytes() {
    // Clear on empty Bytes
    let mut empty_bytes = Bytes::new();
    assert(empty_bytes.len() == 0);
    assert(empty_bytes.capacity() == 0);

    empty_bytes.clear();
    assert(empty_bytes.len() == 0);
    assert(empty_bytes.capacity() == 0);
}

#[test]
fn bytes_is_empty() {
    let (mut setup_bytes, _, _, _) = setup();

    assert(!setup_bytes.is_empty());

    let mut new_bytes = Bytes::new();
    assert(new_bytes.is_empty());

    let mut capacity_bytes = Bytes::with_capacity(16);
    assert(capacity_bytes.is_empty());
}

#[test]
fn bytes_ptr() {
    let (mut setup_bytes, a, _, _) = setup();

    let setup_bytes_ptr = setup_bytes.ptr();
    assert(!setup_bytes_ptr.is_null());
    assert(setup_bytes_ptr.read::<u8>() == a);

    let mut new_bytes = Bytes::new();
    let new_bytes_ptr = new_bytes.ptr();
    assert(!new_bytes_ptr.is_null());

    let mut capacity_bytes = Bytes::with_capacity(16);
    let capacity_bytes_ptr = capacity_bytes.ptr();
    assert(!capacity_bytes_ptr.is_null());
}

#[test()]
fn bytes_split_at() {
    let (mut bytes_1, a, b, c) = setup();
    let d = 7u8;
    bytes_1.push(d);
    assert(bytes_1.len() == 4);

    let index = 2;
    let (bytes_1_left, bytes_1_right) = bytes_1.split_at(index);
    assert(bytes_1.capacity() == 4);
    assert(bytes_1_right.capacity() == 2);
    assert(bytes_1_left.capacity() == 2);
    assert(bytes_1_left.len() == 2);
    assert(bytes_1_right.len() == 2);
    assert(bytes_1_left.get(0).unwrap() == a);
    assert(bytes_1_left.get(1).unwrap() == b);
    assert(bytes_1_right.get(0).unwrap() == c);
    assert(bytes_1_right.get(1).unwrap() == d);
}

#[test()]
fn bytes_split_at_twice() {
    let (mut bytes, a, b, _c) = setup();
    let d = 7u8;
    bytes.push(d);
    assert(bytes.len() == 4);

    let index = 2;
    let (bytes_left, _bytes_right) = bytes.split_at(index);

    // Split twice
    let index_2 = 1;
    let (left_left, left_right) = bytes_left.split_at(index_2);
    assert(bytes_left.capacity() == 2);
    assert(left_left.capacity() == 1);
    assert(left_right.capacity() == 1);
    assert(left_left.len() == 1);
    assert(left_right.len() == 1);
    assert(left_left.get(0).unwrap() == a);
    assert(left_right.get(0).unwrap() == b);
}

#[test()]
fn bytes_split_at_end() {
    // // Split at end
    let (mut bytes, a, b, c) = setup();

    let index = bytes.len();
    let (bytes_left, bytes_right) = bytes.split_at(index);
    assert(bytes.capacity() == 4);
    assert(bytes_left.capacity() == 3);
    assert(bytes_right.capacity() == 0);
    assert(bytes_left.len() == 3);
    assert(bytes_right.len() == 0);
    assert(bytes_left.get(0).unwrap() == a);
    assert(bytes_left.get(1).unwrap() == b);
    assert(bytes_left.get(2).unwrap() == c);
    assert(bytes_right.get(0).is_none());
}

#[test()]
fn bytes_split_at_front() {
    // Split at front
    let (mut bytes, a, b, c) = setup();

    let index = 0;
    let (bytes_left, bytes_right) = bytes.split_at(index);
    assert(bytes.capacity() == 4);
    assert(bytes_left.capacity() == 0);
    assert(bytes_right.capacity() == 3);
    assert(bytes_left.len() == 0);
    assert(bytes_right.len() == 3);
    assert(bytes_right.get(0).unwrap() == a);
    assert(bytes_right.get(1).unwrap() == b);
    assert(bytes_right.get(2).unwrap() == c);
    assert(bytes_left.get(0).is_none());
}

#[test(should_revert)]
fn revert_bytes_split_at_out_of_bounds() {
    let (mut bytes, _a, _b, _c) = setup();

    let (_bytes_left, _bytes_right) = bytes.split_at(bytes.len() + 1);
}

#[test()]
fn bytes_append() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);

    let mut bytes2 = Bytes::new();
    let d = 5u8;
    let e = 7u8;
    let f = 9u8;
    bytes2.push(d);
    bytes2.push(e);
    bytes2.push(f);
    assert(bytes2.len() == 3);
    assert(bytes2.get(0).unwrap() == d);
    assert(bytes2.get(1).unwrap() == e);
    assert(bytes2.get(2).unwrap() == f);

    let first_length = bytes.len();
    let second_length = bytes2.len();
    bytes.append(bytes2);
    assert(bytes.len() == first_length + second_length);
    assert(bytes.capacity() == first_length + first_length);
    assert(bytes2.is_empty());
    let values = [a, b, c, d, e, f];
    let mut i = 0;
    while i < 6 {
        assert(bytes.get(i).unwrap() == values[i]);
        i += 1;
    };
}

#[test()]
fn bytes_append_empty() {
    // Append empty bytes
    let (mut bytes, a, b, c) = setup();
    let bytes_length = bytes.len();
    let bytes_original_capacity = bytes.capacity();

    let mut empty_bytes = Bytes::new();
    bytes.append(empty_bytes);

    // Because empty bytes were appended, no changes to length and capacity were made
    assert(bytes.len() == bytes_length);
    assert(bytes.capacity() == bytes_original_capacity);
    assert(empty_bytes.is_empty());

    let values = [a, b, c];
    let mut i = 0;
    while i < 3 {
        assert(bytes.get(i).unwrap() == values[i]);
        i += 1;
    };
}

#[test()]
fn bytes_append_to_empty() {
    // Append to empty bytes
    let (mut bytes, a, b, c) = setup();
    let bytes_length = bytes.len();
    let bytes_original_capacity = bytes.capacity();

    // Because empty bytes were appended, no changes to capacity were made
    let mut empty_bytes = Bytes::new();
    empty_bytes.append(bytes);
    assert(empty_bytes.len() == bytes_length);
    assert(empty_bytes.capacity() == bytes_original_capacity);
    assert(bytes.is_empty());

    let values = [a, b, c];
    let mut i = 0;
    while i < 3 {
        assert(empty_bytes.get(i).unwrap() == values[i]);
        i += 1;
    };
}

#[test()]
fn bytes_eq() {
    let (mut bytes, _a, _b, _c) = setup();

    let d = 5u8;
    let e = 7u8;
    let f = 9u8;
    let mut other = Bytes::new();
    other.push(d);
    other.push(e);
    other.push(f);
    assert(bytes == other);

    other.push(42u8);
    bytes.push(42u8);
    assert(bytes == other);
}

#[test()]
fn bytes_ne() {
    let (mut bytes, _a, _b, _c) = setup();

    let d = 5u8;
    let e = 7u8;
    let f = 9u8;
    let mut other = Bytes::new();
    other.push(d);
    other.push(e);
    other.push(f);

    other.push(42u8);
    assert(bytes != other);

    other.swap(0, 1);
    assert(bytes != other);
}

#[test()]
fn bytes_as_raw_slice() {
    let (mut bytes, _a, _b, _c) = setup();

    let slice = bytes.as_raw_slice();
    assert(bytes.ptr() == slice.ptr());
    assert(bytes.len() == slice.number_of_bytes());
}

#[test]
fn bytes_from_b256() {
    let initial = 0x3333333333333333333333333333333333333333333333333333333333333333;
    let b: Bytes = Bytes::from(initial);
    let mut control_bytes = Bytes::with_capacity(32);

    let mut i = 0;
    while i < 32 {
        // 0x33 is 51 in decimal
        control_bytes.push(51u8);
        i += 1;
    }

    assert(b == control_bytes);
}

#[test]
fn bytes_into_b256() {
    let mut initial_bytes = Bytes::with_capacity(32);

    let mut i = 0;
    while i < 32 {
        // 0x33 is 51 in decimal
        initial_bytes.push(51u8);
        i += 1;
    }

    let value: b256 = initial_bytes.into();
    let expected: b256 = 0x3333333333333333333333333333333333333333333333333333333333333333;

    assert(value == expected);
}

#[test]
fn bytes_b256_from() {
    let control = 0x3333333333333333333333333333333333333333333333333333333333333333;
    let mut bytes = Bytes::with_capacity(32);

    let mut i = 0;
    while i < 32 {
        // 0x33 is 51 in decimal
        bytes.push(51u8);
        i += 1;
    }

    let result_b256: b256 = b256::from(bytes);

    assert(result_b256 == control);
}

#[test]
fn bytes_b256_into() {
    let initial = 0x3333333333333333333333333333333333333333333333333333333333333333;
    let mut control = Bytes::with_capacity(32);

    let mut i = 0;
    while i < 32 {
        // 0x33 is 51 in decimal
        control.push(51u8);
        i += 1;
    }

    let result_bytes: Bytes = initial.into();

    assert(result_bytes == control);
}

#[test()]
fn bytes_from_raw_slice() {
    let val = 0x3497297632836282349729763283628234972976328362823497297632836282;
    let slice = asm(ptr: (__addr_of(val), 32)) {
        ptr: raw_slice
    };

    let mut bytes = Bytes::from(slice);
    assert(bytes.ptr() != slice.ptr()); // Bytes should own its buffer
    assert(bytes.len() == slice.number_of_bytes());
}

#[test()]
fn bytes_into_raw_slice() {
    let (mut bytes, _a, _b, _c) = setup();

    let slice: raw_slice = bytes.into();

    assert(bytes.ptr() == slice.ptr());
    assert(bytes.len() == slice.number_of_bytes());
}

#[test()]
fn bytes_raw_slice_from() {
    let (mut bytes, _a, _b, _c) = setup();

    let slice: raw_slice = raw_slice::from(bytes);

    assert(bytes.ptr() == slice.ptr());
    assert(bytes.len() == slice.number_of_bytes());
}

#[test()]
fn bytes_raw_slice_into() {
    let val = 0x3497297632836282349729763283628234972976328362823497297632836282;
    let slice = asm(ptr: (__addr_of(val), 32)) {
        ptr: raw_slice
    };

    let bytes: Bytes = slice.into();

    assert(bytes.ptr() != slice.ptr()); // Bytes should own its buffer
    assert(bytes.len() == slice.number_of_bytes());
}

#[test()]
fn bytes_from_vec_u8() {
    let mut vec = Vec::new();
    let (_, a, b, c) = setup();
    vec.push(a);
    vec.push(b);
    vec.push(c);

    let bytes = Bytes::from(vec);

    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
}

#[test()]
fn bytes_into_vec_u8() {
    let (mut bytes, a, b, c) = setup();
    assert(bytes.len() == 3);

    let vec: Vec<u8> = bytes.into();

    assert(vec.len() == 3);
    assert(vec.get(0).unwrap() == a);
    assert(vec.get(1).unwrap() == b);
    assert(vec.get(2).unwrap() == c);
}

#[test()]
fn bytes_vec_u8_from() {
    let (mut bytes, a, b, c) = setup();

    let mut vec: Vec<u8> = Vec::<u8>::from(bytes);

    assert(vec.len() == 3);
    assert(vec.get(0).unwrap() == a);
    assert(vec.get(1).unwrap() == b);
    assert(vec.get(2).unwrap() == c);
}

#[test()]
fn bytes_vec_u8_into() {
    let mut vec = Vec::new();
    let (_, a, b, c) = setup();
    vec.push(a);
    vec.push(b);
    vec.push(c);

    let bytes: Bytes = vec.into();

    assert(bytes.len() == 3);
    assert(bytes.get(0).unwrap() == a);
    assert(bytes.get(1).unwrap() == b);
    assert(bytes.get(2).unwrap() == c);
}

#[test]
fn bytes_clone() {
    let (mut bytes, _a, _b, _c) = setup();

    let cloned_bytes = bytes.clone();

    assert(cloned_bytes.ptr() != bytes.ptr());
    assert(cloned_bytes.len() == bytes.len());
    // Capacity is not cloned
    // assert(cloned_bytes.capacity() == bytes.capacity());
    assert(cloned_bytes.get(0).unwrap() == bytes.get(0).unwrap());
    assert(cloned_bytes.get(1).unwrap() == bytes.get(1).unwrap());
    assert(cloned_bytes.get(2).unwrap() == bytes.get(2).unwrap());
}

#[test]
pub fn test_encode_decode() {
    let initial = 0x3333333333333333333333333333333333333333333333333333333333333333;
    let initial: Bytes = Bytes::from(initial);
    let decoded = abi_decode::<Bytes>(encode(initial));

    assert_eq(decoded, initial);
}

#[test()]
fn bytes_test_packing() {
    let mut bytes = Bytes::new();
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    bytes.push(5u8);
    assert(bytes.len() == 11);
    assert(bytes.capacity() == 16);
}

#[test()]
fn bytes_test_u8_limits() {
    let mut bytes = Bytes::new();
    let max = 255u8;
    let min = 0u8;
    bytes.push(max);
    bytes.push(min);
    bytes.push(max);
    bytes.push(min);
    bytes.push(max);
    bytes.push(min);

    assert(bytes.len() == 6);
    assert(bytes.capacity() == 8);
    assert(bytes.get(0).unwrap() == max);
    assert(bytes.get(1).unwrap() == min);
    assert(bytes.get(2).unwrap() == max);
    assert(bytes.get(3).unwrap() == min);
    assert(bytes.get(4).unwrap() == max);
    assert(bytes.get(5).unwrap() == min);
}
