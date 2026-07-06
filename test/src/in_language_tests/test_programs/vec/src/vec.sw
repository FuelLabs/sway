library;

fn setup() -> (Vec<u64>, u64, u64, u64) {
    let mut vec: Vec<u64> = Vec::new();
    let a = 5u64;
    let b = 7u64;
    let c = 9u64;
    vec.push(a);
    vec.push(b);
    vec.push(c);
    (vec, a, b, c)
}

#[test]
fn vec_new() {
    let new_vec: Vec<u64> = Vec::new();
    assert_eq(new_vec.len(), 0);
    assert_eq(new_vec.capacity(), 0);
}

#[test]
fn vec_with_capacity() {
    let vec_1: Vec<u64> = Vec::with_capacity(0);
    assert_eq(vec_1.capacity(), 0);

    let vec_2: Vec<u64> = Vec::with_capacity(1);
    assert_eq(vec_2.capacity(), 1);

    // 2^6
    let vec_3: Vec<u64> = Vec::with_capacity(64);
    assert_eq(vec_3.capacity(), 64);

    // 2^11
    let vec_4: Vec<u64> = Vec::with_capacity(2048);
    assert_eq(vec_4.capacity(), 2048);

    // 2^16
    let vec_5: Vec<u64> = Vec::with_capacity(65536);
    assert_eq(vec_5.capacity(), 65536);
}

#[test()]
fn vec_push() {
    let mut vec: Vec<u64> = Vec::new();

    assert_eq(vec.len(), 0);
    assert_eq(vec.capacity(), 0);

    vec.push(1u64);
    assert_eq(vec.len(), 1);
    assert_eq(vec.capacity(), 1);

    vec.push(2u64);
    assert_eq(vec.len(), 2);
    assert_eq(vec.capacity(), 2);

    // Capacity doubles
    vec.push(3u64);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);

    vec.push(4u64);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);

    // Capacity doubles
    vec.push(5u64);
    assert_eq(vec.len(), 5);
    assert_eq(vec.capacity(), 8);

    vec.push(6u64);
    assert_eq(vec.len(), 6);
    assert_eq(vec.capacity(), 8);

    vec.push(7u64);
    assert_eq(vec.len(), 7);
    assert_eq(vec.capacity(), 8);

    vec.push(8u64);
    assert_eq(vec.len(), 8);
    assert_eq(vec.capacity(), 8);

    // Capacity doubles
    vec.push(9u64);
    assert_eq(vec.len(), 9);
    assert_eq(vec.capacity(), 16);
}

#[test()]
fn vec_capacity() {
    let mut vec: Vec<u64> = Vec::new();
    assert_eq(vec.capacity(), 0);

    vec.push(5u64);
    assert_eq(vec.capacity(), 1);
    vec.push(7u64);
    assert_eq(vec.capacity(), 2);
    vec.push(9u64);
    assert_eq(vec.capacity(), 4);
    vec.push(11u64);
    assert_eq(vec.capacity(), 4);
    vec.push(3u64);
    assert_eq(vec.capacity(), 8);
}

#[test()]
fn vec_clear() {
    let (mut vec, _, _, _) = setup();
    assert_eq(vec.len(), 3);

    vec.clear();
    assert_eq(vec.len(), 0);
}

#[test()]
fn vec_clear_twice() {
    let (mut vec, _, _, _) = setup();

    vec.clear();
    assert_eq(vec.len(), 0);

    // Can clean twice
    vec.push(1u64);
    vec.clear();
    assert_eq(vec.len(), 0);
}

#[test()]
fn vec_clear_empty_vec() {
    // Clear on empty vec
    let mut empty_vec: Vec<u64> = Vec::new();
    assert_eq(empty_vec.len(), 0);
    assert_eq(empty_vec.capacity(), 0);

    empty_vec.clear();
    assert_eq(empty_vec.len(), 0);
    assert_eq(empty_vec.capacity(), 0);
}

#[test()]
fn vec_get() {
    let (vec, a, b, c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), b);
    assert_eq(vec.get(2).unwrap(), c);
    // get is non-modifying
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), b);
    assert_eq(vec.get(2).unwrap(), c);
    assert_eq(vec.len(), 3);

    // None if out of bounds
    assert(vec.get(vec.len()).is_none());
}

#[test()]
fn vec_len() {
    let (mut vec, _, _, _) = setup();
    assert_eq(vec.len(), 3);

    vec.push(5u64);
    assert_eq(vec.len(), 4);
    vec.push(6u64);
    assert_eq(vec.len(), 5);
    vec.push(7u64);
    assert_eq(vec.len(), 6);
    vec.push(8u64);
    assert_eq(vec.len(), 7);
}

#[test]
fn vec_is_empty() {
    let (mut setup_vec, _, _, _) = setup();

    assert(!setup_vec.is_empty());

    let mut new_vec: Vec<u64> = Vec::new();
    assert(new_vec.is_empty());

    let mut capacity_vec: Vec<u64> = Vec::with_capacity(16);
    assert(capacity_vec.is_empty());
}

#[test()]
fn vec_remove() {
    let (mut vec, a, b, c) = setup();
    let d = 7u64;
    vec.push(d);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);

    // Remove in the middle
    let item1 = vec.remove(1);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    assert_eq(item1, b);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), c);
    assert_eq(vec.get(2).unwrap(), d);
    assert(vec.get(3).is_none());
}

#[test()]
fn vec_remove_front() {
    let (mut vec, a, b, c) = setup();
    // Remove at the start
    let item = vec.remove(0);
    assert_eq(vec.len(), 2);
    assert_eq(vec.capacity(), 4);
    assert_eq(item, a);
    assert_eq(vec.get(0).unwrap(), b);
    assert_eq(vec.get(1).unwrap(), c);
    assert(vec.get(2).is_none());
}

#[test()]
fn vec_remove_end() {
    let (mut vec, a, b, c) = setup();
    // Remove at the end
    let item = vec.remove(vec.len() - 1);
    assert_eq(vec.len(), 2);
    assert_eq(vec.capacity(), 4);
    assert_eq(item, c);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), b);
    assert(vec.get(2).is_none());
}

#[test()]
fn vec_remove_all() {
    let (mut vec, a, b, c) = setup();
    // Remove all
    let item1 = vec.remove(0);
    let item2 = vec.remove(0);
    let item3 = vec.remove(0);
    assert_eq(vec.len(), 0);
    assert_eq(vec.capacity(), 4);
    assert_eq(item1, a);
    assert_eq(item2, b);
    assert_eq(item3, c);
    assert(vec.get(0).is_none());
}

#[test(should_revert)]
fn revert_vec_remove_out_of_bounds() {
    let (mut vec, _a, _b, _c) = setup();

    let _result = vec.remove(vec.len());
}

#[test()]
fn vec_insert() {
    let (mut vec, a, b, c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);

    let d = 11u64;

    // Inserts in the middle
    vec.insert(1, d);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), d);
    assert_eq(vec.get(2).unwrap(), b);
    assert_eq(vec.get(3).unwrap(), c);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);
}

#[test()]
fn vec_insert_twice() {
    let (mut vec, a, b, c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);

    let d = 11u64;
    let e = 13u64;

    // Inserts in the middle
    vec.insert(1, d);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), d);
    assert_eq(vec.get(2).unwrap(), b);
    assert_eq(vec.get(3).unwrap(), c);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);

    // Twice
    vec.insert(1, e);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), e);
    assert_eq(vec.get(2).unwrap(), d);
    assert_eq(vec.get(3).unwrap(), b);
    assert_eq(vec.get(4).unwrap(), c);
    assert_eq(vec.len(), 5);
    assert_eq(vec.capacity(), 8);
}

#[test()]
fn vec_insert_front() {
    let (mut vec, a, b, c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);

    let d = 11u64;

    // Inserts at the front
    vec.insert(0, d);
    assert_eq(vec.get(0).unwrap(), d);
    assert_eq(vec.get(1).unwrap(), a);
    assert_eq(vec.get(2).unwrap(), b);
    assert_eq(vec.get(3).unwrap(), c);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);
}

#[test()]
fn vec_insert_before_back() {
    let (mut vec, a, b, c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);

    let d = 11u64;

    // Inserts right before the back
    vec.insert(vec.len() - 1, d);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), b);
    assert_eq(vec.get(2).unwrap(), d);
    assert_eq(vec.get(3).unwrap(), c);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);
}

#[test()]
fn vec_insert_back() {
    let (mut vec, a, b, c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);

    let d = 11u64;

    // Inserts at the back
    vec.insert(vec.len(), d);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), b);
    assert_eq(vec.get(2).unwrap(), c);
    assert_eq(vec.get(3).unwrap(), d);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);
}

#[test(should_revert)]
fn revert_vec_insert_out_of_bounds() {
    let (mut vec, a, _b, _c) = setup();

    vec.insert(vec.len() + 1, a);
}

#[test()]
fn vec_pop() {
    let (mut vec, a, b, c) = setup();
    assert_eq(vec.len(), 3);

    vec.push(42u64);
    vec.push(11u64);
    vec.push(69u64);
    vec.push(100u64);
    vec.push(200u64);
    vec.push(255u64);
    vec.push(180u64);
    vec.push(17u64);
    vec.push(19u64);
    assert_eq(vec.len(), 12);
    assert_eq(vec.capacity(), 16);

    let first = vec.pop();
    assert_eq(first.unwrap(), 19u64);
    assert_eq(vec.len(), 11);
    assert_eq(vec.capacity(), 16);

    let second = vec.pop();
    assert_eq(second.unwrap(), 17u64);
    assert_eq(vec.len(), 10);
    assert_eq(vec.capacity(), 16);

    let third = vec.pop();
    assert_eq(third.unwrap(), 180u64);
    assert_eq(vec.len(), 9);
    let _ = vec.pop();
    let _ = vec.pop();
    let _ = vec.pop();
    let _ = vec.pop();
    let _ = vec.pop();
    let _ = vec.pop();
    assert_eq(vec.len(), 3);
    assert_eq(vec.pop().unwrap(), c);
    assert_eq(vec.pop().unwrap(), b);
    assert_eq(vec.pop().unwrap(), a);

    // Can pop all
    assert_eq(vec.len(), 0);
    assert_eq(vec.capacity(), 16);
    assert(vec.pop().is_none());
}

#[test()]
fn vec_swap() {
    let (mut vec, a, b, c) = setup();
    let d = 5u64;
    vec.push(d);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);

    // Swaps Middle
    vec.swap(1, 2);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), c);
    assert_eq(vec.get(2).unwrap(), b);
    assert_eq(vec.get(3).unwrap(), d);
}

#[test()]
fn vec_swap_twice() {
    let (mut vec, a, b, c) = setup();
    let d = 5u64;
    vec.push(d);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);

    // Swaps Middle
    vec.swap(1, 2);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), c);
    assert_eq(vec.get(2).unwrap(), b);
    assert_eq(vec.get(3).unwrap(), d);

    vec.swap(1, 2);
    assert_eq(vec.len(), 4);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), b);
    assert_eq(vec.get(2).unwrap(), c);
    assert_eq(vec.get(3).unwrap(), d);
}

#[test()]
fn vec_swap_front() {
    let (mut vec, a, b, c) = setup();

    // Swaps Front
    vec.swap(0, 1);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), b);
    assert_eq(vec.get(1).unwrap(), a);
    assert_eq(vec.get(2).unwrap(), c);
}

#[test()]
fn vec_swap_end() {
    let (mut vec, a, b, c) = setup();

    // Swaps back
    vec.swap(2, 1);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), c);
    assert_eq(vec.get(2).unwrap(), b);
}

#[test()]
fn vec_swap_front_with_end() {
    let (mut vec, a, b, c) = setup();

    // Swaps front with back
    vec.swap(0, 2);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), c);
    assert_eq(vec.get(1).unwrap(), b);
    assert_eq(vec.get(2).unwrap(), a);
}

#[test(should_revert)]
fn revert_vec_swap_element_1_out_of_bounds() {
    let (mut vec, _a, _b, _c) = setup();

    vec.swap(vec.len(), 0);
}

#[test(should_revert)]
fn revert_vec_swap_element_2_out_of_bounds() {
    let (mut vec, _a, _b, _c) = setup();

    vec.swap(0, vec.len());
}

#[test()]
fn vec_set() {
    let (mut vec, a, _b, c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    let d = 11u64;

    // Sets in the middle
    vec.set(1, d);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), d);
    assert_eq(vec.get(2).unwrap(), c);
}

#[test()]
fn vec_set_twice() {
    let (mut vec, a, _b, c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    let d = 11u64;
    let e = 13u64;

    // Sets in the middle
    vec.set(1, d);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), d);
    assert_eq(vec.get(2).unwrap(), c);

    // Twice
    vec.set(1, e);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), e);
    assert_eq(vec.get(2).unwrap(), c);
}

#[test()]
fn vec_set_front() {
    let (mut vec, _a, b, c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    let d = 11u64;

    // Sets at the front
    vec.set(0, d);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), d);
    assert_eq(vec.get(1).unwrap(), b);
    assert_eq(vec.get(2).unwrap(), c);
}

#[test()]
fn vec_set_back() {
    let (mut vec, a, b, _c) = setup();
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    let d = 11u64;

    // Sets at the back
    vec.set(vec.len() - 1, d);
    assert_eq(vec.len(), 3);
    assert_eq(vec.capacity(), 4);
    assert_eq(vec.get(0).unwrap(), a);
    assert_eq(vec.get(1).unwrap(), b);
    assert_eq(vec.get(2).unwrap(), d);
}

#[test(should_revert)]
fn revert_vec_set_out_of_bounds() {
    let (mut vec, _a, _b, _c) = setup();

    vec.set(vec.len(), 11u64);
}

#[test]
fn vec_iter() {
    let mut vector: Vec<u64> = Vec::new();

    let number0 = 0;
    let number1 = 1;
    let number2 = 2;
    let number3 = 3;
    let number4 = 4;

    vector.push(number0);
    vector.push(number1);
    vector.push(number2);
    vector.push(number3);
    vector.push(number4);

    let mut iter = vector.iter();

    assert_eq(iter.next(), Some(number0));
    assert_eq(iter.next(), Some(number1));
    assert_eq(iter.next(), Some(number2));
    assert_eq(iter.next(), Some(number3));
    assert_eq(iter.next(), Some(number4));
    assert_eq(iter.next(), None);
    assert_eq(iter.next(), None);
}

#[test]
fn vec_ptr() {
    let (mut setup_vec, a, _, _) = setup();

    let setup_vec_ptr = setup_vec.ptr();
    assert(!setup_vec_ptr.is_null());
    assert_eq(setup_vec_ptr.read::<u64>(), a);

    let mut new_vec: Vec<u64> = Vec::new();
    let new_vec_ptr = new_vec.ptr();
    assert(!new_vec_ptr.is_null());

    let mut capacity_vec: Vec<u64> = Vec::with_capacity(16);
    let capacity_vec_ptr = capacity_vec.ptr();
    assert(!capacity_vec_ptr.is_null());
}

#[test()]
fn vec_as_raw_slice() {
    let (mut vec, _a, _b, _c) = setup();

    let slice = vec.as_raw_slice();
    assert(vec.ptr() == slice.ptr());
    assert_eq(vec.len(), slice.len::<u64>());
}

#[test()]
fn vec_from_raw_slice() {
    let val = 0x3497297632836282349729763283628234972976328362823497297632836282;
    let slice = asm(ptr: (__addr_of(val), 32)) {
        ptr: raw_slice
    };

    let mut vec: Vec<u64> = Vec::from(slice);
    assert(vec.ptr() != slice.ptr()); // Vec should own its buffer
    assert_eq(vec.len(), slice.len::<u64>());
}

// TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6085 is resolved
// #[test()]
// fn vec_into_raw_slice() {
//     // Glob operator needed for From<Vec> for raw_slice
//     use std::vec::*;

//     let (mut vec, _a, _b, _c) = setup();

//     let slice: raw_slice = vec.into();

//     assert_eq(vec.ptr(), slice.ptr());
//     assert_eq(vec.len(), slice.len::<u64>());
// }

// TODO: Uncomment when https://github.com/FuelLabs/sway/issues/6085 is resolved
// #[test()]
// fn vec_raw_slice_from() {
//     // Glob operator needed for From<Vec> for raw_slice
//     use std::vec::*;

//     let (mut vec, _a, _b, _c) = setup();

//     let slice: raw_slice = <raw_slice as From<Vec<T>>>::from(vec);

//     assert_eq(vec.ptr(), slice.ptr());
//     assert_eq(vec.len(), slice.len::<u64>());
// }


#[test()]
fn vec_raw_slice_into() {
    let val = 0x3497297632836282349729763283628234972976328362823497297632836282;
    let slice = asm(ptr: (__addr_of(val), 32)) {
        ptr: raw_slice
    };

    let vec: Vec<u64> = slice.into();

    assert(vec.ptr() != slice.ptr()); // Vec should own its buffer
    assert_eq(vec.len(), slice.len::<u64>());
}

#[test]
fn vec_clone() {
    let (mut vec, _a, _b, _c) = setup();

    let cloned_vec = vec.clone();

    assert(cloned_vec.ptr() != vec.ptr());
    assert_eq(cloned_vec.len(), vec.len());
    // Capacity is not cloned
    assert_ne(cloned_vec.capacity(), vec.capacity());
    assert_eq(cloned_vec.get(0).unwrap(), vec.get(0).unwrap());
    assert_eq(cloned_vec.get(1).unwrap(), vec.get(1).unwrap());
    assert_eq(cloned_vec.get(2).unwrap(), vec.get(2).unwrap());
}

#[test]
fn vec_buffer_ownership() {
    let mut original_array = [1u8, 2u8, 3u8, 4u8];
    let slice = raw_slice::from_parts::<u8>(__addr_of(original_array), 4);

    // Check Vec duplicates the original slice
    let mut bytes = Vec::<u8>::from(slice);
    bytes.set(0, 5);
    assert_eq(original_array[0], 1);

    // At this point, slice equals [5, 2, 3, 4]
    let encoded_slice = encode(bytes);

    // `Vec<u8>` should duplicate the underlying buffer,
    // so when we write to it, it should not change
    // `encoded_slice` 
    let mut bytes = abi_decode::<Vec<u8>>(encoded_slice);
    bytes.set(0, 6);
    assert_eq(bytes.get(0), Some(6));

    let mut bytes = abi_decode::<Vec<u8>>(encoded_slice);
    assert_eq(bytes.get(0), Some(5));
}

#[test()]
fn vec_encode_and_decode() {
    let mut v1: Vec<u64> = Vec::new();
    v1.push(1);
    v1.push(2);
    v1.push(3);

    let v2 = abi_decode::<Vec<u64>>(encode(v1));

    assert_eq(v2.len(), 3);
    assert_eq(v2.capacity(), 3);
    assert_eq(v2.get(0), Some(1));
    assert_eq(v2.get(1), Some(2));
    assert_eq(v2.get(2), Some(3));
}

#[test]
fn vec_resize() {
    let (mut vec_1, _, _, _) = setup();
    assert_eq(vec_1.len(), 3);
    assert_eq(vec_1.capacity(), 4);

    // Resize to same size, no effect
    vec_1.resize(3, 0);
    assert_eq(vec_1.len(), 3);
    assert_eq(vec_1.capacity(), 4);

    // Resize to capacity size doesn't impact capacity
    vec_1.resize(4, 1);
    assert_eq(vec_1.len(), 4);
    assert_eq(vec_1.capacity(), 4);
    assert_eq(vec_1.get(0), Some(5));
    assert_eq(vec_1.get(1), Some(7));
    assert_eq(vec_1.get(2), Some(9));
    assert_eq(vec_1.get(3), Some(1));

    // Resize increases size and capacity
    vec_1.resize(10, 2);
    assert_eq(vec_1.len(), 10);
    assert_eq(vec_1.capacity(), 10);
    assert_eq(vec_1.get(0), Some(5));
    assert_eq(vec_1.get(1), Some(7));
    assert_eq(vec_1.get(2), Some(9));
    assert_eq(vec_1.get(3), Some(1));
    assert_eq(vec_1.get(4), Some(2));
    assert_eq(vec_1.get(5), Some(2));
    assert_eq(vec_1.get(6), Some(2));
    assert_eq(vec_1.get(7), Some(2));
    assert_eq(vec_1.get(8), Some(2));
    assert_eq(vec_1.get(9), Some(2));

    // Resize to less doesn't impact capacity or order
    vec_1.resize(1, 0);
    assert_eq(vec_1.len(), 1);
    assert_eq(vec_1.capacity(), 10);
    assert_eq(vec_1.get(0), Some(5));
    assert_eq(vec_1.get(1), None);

    // Resize to zero doesn't impact capacity and returns None
    vec_1.resize(0, 0);
    assert_eq(vec_1.len(), 0);
    assert_eq(vec_1.capacity(), 10);
    assert_eq(vec_1.get(0), None);

    let mut vec_2 = Vec::new();

    // Resize to zero on empty vec doesn't impact
    vec_2.resize(0, 0);
    assert_eq(vec_2.len(), 0);
    assert_eq(vec_2.capacity(), 0);

    // Resize on empty vec fills and sets capacity
    vec_2.resize(3, 1);
    assert_eq(vec_2.len(), 3);
    assert_eq(vec_2.capacity(), 3);
    assert_eq(vec_2.get(0), Some(1));
    assert_eq(vec_2.get(1), Some(1));
    assert_eq(vec_2.get(2), Some(1));
}

#[test]
fn vec_last() {
    let (mut vec_1, _, _, _) = setup();
    assert_eq(vec_1.last(), Some(9));

    vec_1.push(2);
    assert_eq(vec_1.last(), Some(2));

    vec_1.push(3);
    assert_eq(vec_1.last(), Some(3));

    let _ = vec_1.pop();
    assert_eq(vec_1.last(), Some(2));

    let vec_2: Vec<u64> = Vec::new();
    assert_eq(vec_2.last(), None)
}

#[allow(dead_code)]
fn create_vec_trivial(n: u64) -> Vec<u64> {
    let mut vec = Vec::<u64>::new();
    let mut i: u64 = 0;
    while i < n {
        vec.push(i);
        i += 1;
    }
    vec
}

#[allow(dead_code)]
fn create_nested_vec_trivial(n: u64) -> Vec<Vec<u64>> {
    let mut vec = Vec::<Vec<u64>>::new();
    let mut i: u64 = 0;
    while i < n {
        vec.push(create_vec_trivial(i));
        i += 1;
    }
    vec
}

#[allow(dead_code)]
fn create_vec_non_trivial(n: u32) -> Vec<u32> {
    let mut vec = Vec::<u32>::new();
    let mut i: u32 = 0;
    while i < n {
        vec.push(i);
        i += 1;
    }
    vec
}

#[allow(dead_code)]
fn create_nested_vec_non_trivial(n: u32) -> Vec<Vec<u32>> {
    let mut vec = Vec::<Vec<u32>>::new();
    let mut i: u32 = 0;
    while i < n {
        vec.push(create_vec_non_trivial(i));
        i += 1;
    }
    vec
}

#[allow(dead_code)]
fn create_vec_zst(n: u32) -> Vec<()> {
    let mut vec = Vec::<()>::new();
    let mut i: u32 = 0;
    while i < n {
        vec.push(());
        i += 1;
    }
    vec
}

#[allow(dead_code)]
fn create_nested_vec_zst(n: u32) -> Vec<Vec<()>> {
    let mut vec = Vec::<Vec<()>>::new();
    let mut i: u32 = 0;
    while i < n {
        vec.push(create_vec_zst(i));
        i += 1;
    }
    vec
}

#[test]
fn vec_encode_decode_trivial() {
    let vec = create_vec_trivial(10);
    let encoded_decoded: Vec<u64> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<u64> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}

#[test]
fn vec_encode_decode_nested_trivial() {
    let vec = create_nested_vec_trivial(10);
    let encoded_decoded: Vec<Vec<u64>> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<Vec<u64>> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}

#[test]
fn vec_encode_decode_non_trivial() {
    let vec = create_vec_non_trivial(10);
    let encoded_decoded: Vec<u32> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<u32> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}

#[test]
fn vec_encode_decode_nested_non_trivial() {
    let vec = create_nested_vec_non_trivial(10);
    let encoded_decoded: Vec<Vec<u32>> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<Vec<u32>> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}

#[test]
fn vec_encode_decode_zst() {
    let vec = create_vec_zst(10);
    let encoded_decoded: Vec<()> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<()> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}

#[test]
fn vec_encode_decode_nested_zst() {
    let vec = create_nested_vec_zst(10);
    let encoded_decoded: Vec<Vec<()>> = abi_decode(encode(vec));
    assert_eq(encoded_decoded, vec);

    let encoded_as_alias = encode_allow_alias(&encoded_decoded);
    let encoded_decoded: Vec<Vec<()>> = abi_decode(encoded_as_alias);
    assert_eq(encoded_decoded, vec);

    log(encoded_decoded);
}
