library;

use std::{bytes::Bytes, string::String};

#[test]
fn string_as_bytes() {
    let mut string = String::new();

    let bytes = string.as_bytes();
    assert_eq(bytes.len(), 0);
    assert_eq(bytes.capacity(), string.capacity());
    assert_eq(bytes.len(), string.len());

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);

    let bytes = string.as_bytes();
    assert_eq(bytes.len(), 1);
    assert_eq(bytes.capacity(), string.capacity());
    assert_eq(bytes.len(), string.len());
}

#[test]
fn string_capacity() {
    let mut string = String::new();
    assert_eq(string.capacity(), 0);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    assert_eq(string.capacity(), 1);
}

#[test]
fn string_len() {
    let mut string = String::new();
    assert_eq(string.len(), 0);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    assert_eq(string.len(), 1);

    let mut string = String::from_ascii_str("ABCDEF");
    assert_eq(string.len(), 6);
}

#[test]
fn string_clear() {
    // Clear non-empty
    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let mut string = String::from_ascii(bytes);
    assert(!string.is_empty());
    assert_eq(string.len(), 1);

    string.clear();
    assert(string.is_empty());
    assert_eq(string.len(), 0);
}

#[test]
fn string_clear_empty() {
    let mut string = String::new();

    assert(string.is_empty());
    assert_eq(string.len(), 0);

    string.clear();

    assert(string.is_empty());
    assert_eq(string.len(), 0);
}

#[test]
fn string_from_ascii_empty() {
    let bytes = Bytes::new();

    // Dummy allocation so that empty `String` allocation
    // does not overlap with the empty `bytes` (`hp` does not
    // move on empty allocation).
    let _ = std::alloc::alloc_bytes(1);

    let string = String::from_ascii(bytes);
    assert_eq(bytes.len(), string.capacity());
    assert_eq(bytes.len(), string.len());
    assert_eq(string.len(), 0);

    assert(string.ptr() != bytes.ptr()); // String creates its own buffer
}

#[test]
fn string_from_ascii() {
    let mut bytes = Bytes::new();
    bytes.push(65u8);
    bytes.push(66u8);
    bytes.push(67u8);
    bytes.push(68u8);
    bytes.push(69u8);

    let string = String::from_ascii(bytes);
    assert_eq(bytes.len(), string.capacity());
    assert_eq(bytes.len(), string.len());

    assert(string.ptr() != bytes.ptr()); // String creates its own buffer

    let string_bytes = string.as_bytes();
    assert_eq(string_bytes.get(0).unwrap(), 65u8);
    assert_eq(string_bytes.get(1).unwrap(), 66u8);
    assert_eq(string_bytes.get(2).unwrap(), 67u8);
    assert_eq(string_bytes.get(3).unwrap(), 68u8);
    assert_eq(string_bytes.get(4).unwrap(), 69u8);
    assert_eq(string_bytes.get(5), None);
}

#[test]
fn string_from_moved_ascii_empty() {
    let bytes = Bytes::new();

    // Dummy allocation to make sure `String` will point
    // to the original empty `bytes`, because it is not allocating
    // on it's own.
    let _ = std::alloc::alloc_bytes(1);

    let string = String::from_moved_ascii(bytes);
    assert_eq(bytes.len(), string.capacity());
    assert_eq(bytes.len(), string.len());
    assert_eq(string.len(), 0);

    assert(string.ptr() == bytes.ptr());  // String takes ownership of the bytes buffer
}

#[test]
fn string_from_moved_ascii() {
    let mut bytes = Bytes::new();
    bytes.push(65u8);
    bytes.push(66u8);
    bytes.push(67u8);
    bytes.push(68u8);
    bytes.push(69u8);

    let bytes_ptr = bytes.ptr();
    let bytes_len = bytes.len();
    let bytes_capacity = bytes.capacity();

    let string = String::from_moved_ascii(bytes);

    assert(string.ptr() == bytes_ptr); // String takes ownership of the bytes buffer
    assert_eq(string.len(), bytes_len);
    assert_eq(string.capacity(), bytes_capacity);

    let string_bytes = string.as_bytes();
    assert_eq(string_bytes.get(0).unwrap(), 65u8);
    assert_eq(string_bytes.get(1).unwrap(), 66u8);
    assert_eq(string_bytes.get(2).unwrap(), 67u8);
    assert_eq(string_bytes.get(3).unwrap(), 68u8);
    assert_eq(string_bytes.get(4).unwrap(), 69u8);
    assert_eq(string_bytes.get(5), None);
}

#[test]
fn string_from_ascii_str_empty() {
    let string = String::from_ascii_str("");
    assert_eq(string.capacity(), 0);
    assert_eq(string.len(), 0);
}

#[test]
fn string_from_ascii_str() {
    let ascii_str = "ABCDEF";
    let ascii_str_ptr = __transmute::<str, raw_slice>(ascii_str).ptr();

    let string = String::from_ascii_str(ascii_str);
    assert_eq(string.capacity(), 6);
    assert_eq(string.len(), 6);

    assert(string.ptr() != ascii_str_ptr); // String creates its own buffer

    let bytes = string.as_bytes();

    assert_eq(bytes.get(0).unwrap(), 65u8);
    assert_eq(bytes.get(1).unwrap(), 66u8);
    assert_eq(bytes.get(2).unwrap(), 67u8);
    assert_eq(bytes.get(3).unwrap(), 68u8);
    assert_eq(bytes.get(4).unwrap(), 69u8);
    assert_eq(bytes.get(5).unwrap(), 70u8);
    assert(bytes.get(6).is_none());
}

#[test]
fn string_from_ascii_str_array_empty() {
    let string = String::from_ascii_str_array(__to_str_array(""));
    assert_eq(string.capacity(), 0);
    assert_eq(string.len(), 0);
}

#[test]
fn string_from_ascii_str_array() {
    let ascii_str = __to_str_array("ABCDEF");
    let ascii_str_ptr = __transmute::<(raw_ptr, u64), raw_slice>((__addr_of(ascii_str), 6)).ptr();

    let string = String::from_ascii_str_array(ascii_str);
    assert_eq(string.capacity(), 6);
    assert_eq(string.len(), 6);

    assert(string.ptr() != ascii_str_ptr); // String creates its own buffer

    let bytes = string.as_bytes();

    assert_eq(bytes.get(0).unwrap(), 65u8);
    assert_eq(bytes.get(1).unwrap(), 66u8);
    assert_eq(bytes.get(2).unwrap(), 67u8);
    assert_eq(bytes.get(3).unwrap(), 68u8);
    assert_eq(bytes.get(4).unwrap(), 69u8);
    assert_eq(bytes.get(5).unwrap(), 70u8);
    assert(bytes.get(6).is_none());
}

#[test]
fn string_from_shared_ascii_str_empty() {
    let string = String::from_shared_ascii_str("");
    assert_eq(string.capacity(), 0);
    assert_eq(string.len(), 0);
}

#[test]
fn string_from_shared_ascii_str() {
    let ascii_str = "ABCDEF";
    let ascii_str_ptr = __transmute::<str, raw_slice>(ascii_str).ptr();

    let string = String::from_shared_ascii_str(ascii_str);

    assert(string.ptr() == ascii_str_ptr); // Shared String uses the original string slice
    assert_eq(string.len(), 6);
    assert_eq(string.capacity(), 6);

    let bytes = string.as_bytes();
    assert_eq(bytes.get(0).unwrap(), 65u8);
    assert_eq(bytes.get(1).unwrap(), 66u8);
    assert_eq(bytes.get(2).unwrap(), 67u8);
    assert_eq(bytes.get(3).unwrap(), 68u8);
    assert_eq(bytes.get(4).unwrap(), 69u8);
    assert_eq(bytes.get(5).unwrap(), 70u8);
    assert(bytes.get(6).is_none());
}

#[test(should_revert)]
fn string_from_shared_ascii_str_immutable_static_str() {
    let string = String::from_shared_ascii_str("static str");
    let mut bytes = Bytes::from_moved_raw_slice(string.as_raw_slice());
    bytes.set(0, 71); // Reverts, because the "static str" is read-only (data section).
}

#[test]
fn string_from_shared_ascii_str_array_empty() {
    let string = String::from_shared_ascii_str_array(__to_str_array(""));
    assert_eq(string.capacity(), 0);
    assert_eq(string.len(), 0);
}

#[test]
fn string_from_shared_ascii_str_array() {
    let string_array = __to_str_array("ABCDEF");
    let string = String::from_shared_ascii_str_array(string_array);

    // TODO: Enable this test once https://github.com/FuelLabs/sway/issues/7681 is fixed.
    //       The below assert passes in `release` mode, but fails in `debug`.
    //       The issue is likely related to the #7681.

    // assert(string.ptr() == __addr_of(string_array)); // Shared String uses the original string array
    assert_eq(string.len(), 6);
    assert_eq(string.capacity(), 6);

    let bytes = string.as_bytes();
    assert_eq(bytes.get(0).unwrap(), 65u8);
    assert_eq(bytes.get(1).unwrap(), 66u8);
    assert_eq(bytes.get(2).unwrap(), 67u8);
    assert_eq(bytes.get(3).unwrap(), 68u8);
    assert_eq(bytes.get(4).unwrap(), 69u8);
    assert_eq(bytes.get(5).unwrap(), 70u8);
    assert(bytes.get(6).is_none());
}

#[test]
fn string_is_empty() {
    let mut string = String::new();
    assert(string.is_empty());
    assert_eq(string.len(), 0);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    assert(!string.is_empty());
    assert_eq(string.len(), 1);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.push(1u8);
    let mut string = String::from_ascii(bytes);
    assert(!string.is_empty());
    assert_eq(string.len(), 2);

    string.clear();
    assert(string.is_empty());
    assert_eq(string.len(), 0);
}

#[test]
fn string_new() {
    let mut string = String::new();

    assert(string.is_empty());
    assert_eq(string.capacity(), 0);
    assert_eq(string.len(), 0);
}

#[test]
fn string_with_capacity() {
    let mut iterator = 0;

    while iterator < 16 {
        let mut string = String::with_capacity(iterator);
        assert_eq(string.capacity(), iterator);
        iterator += 1;
    }

    let mut string = String::with_capacity(0);
    assert_eq(string.capacity(), 0);
    assert_eq(string.len(), 0);

    string.clear();
    assert_eq(string.capacity(), 0);
    assert_eq(string.len(), 0);

    let mut string = String::with_capacity(4);
    assert_eq(string.capacity(), 4);
    assert_eq(string.len(), 0);
}

#[test]
fn string_ptr() {
    let string = String::new();
    assert(!string.ptr().is_null());

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let mut string = String::from_ascii(bytes);
    assert(!string.ptr().is_null());
    assert(string.ptr() != bytes.ptr());
}

#[test]
fn string_from_bytes() {
    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let mut string_from_bytes = String::from(bytes);
    let bytes = string_from_bytes.as_bytes();
    assert_eq(bytes.len(), 5);
    assert_eq(string_from_bytes.len(), 5);
    assert_eq(bytes.capacity(), string_from_bytes.capacity());
    assert_eq(bytes.get(0).unwrap(), 0u8);
    assert_eq(bytes.get(1).unwrap(), 1u8);
    assert_eq(bytes.get(2).unwrap(), 2u8);
}

#[test]
fn string_into_bytes() {
    let mut string = String::new();

    let bytes: Bytes = string.into();
    assert_eq(bytes.len(), 0);
    assert_eq(bytes.capacity(), string.capacity());

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    let bytes: Bytes = string.into();
    assert_eq(bytes.len(), 1);
    assert_eq(bytes.capacity(), string.capacity());
    assert_eq(bytes.get(0).unwrap(), 0u8);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.push(1u8);
    let string = String::from_ascii(bytes);
    let mut bytes: Bytes = string.into();
    assert_eq(bytes.len(), 2);
    assert_eq(bytes.capacity(), string.capacity());
    assert_eq(bytes.get(1).unwrap(), 1u8);
}

#[test]
fn string_bytes_from() {
    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let mut string_from_bytes = String::from(bytes);

    let bytes = Bytes::from(string_from_bytes);
    assert_eq(bytes.len(), 5);
    assert_eq(bytes.capacity(), string_from_bytes.capacity());
    assert_eq(bytes.get(0).unwrap(), 0u8);
    assert_eq(bytes.get(1).unwrap(), 1u8);
    assert_eq(bytes.get(2).unwrap(), 2u8);
}

#[test]
fn string_bytes_into() {
    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let mut string_from_bytes: String = bytes.into();

    let bytes: Bytes = string_from_bytes.as_bytes();
    assert_eq(bytes.len(), 5);
    assert_eq(bytes.capacity(), string_from_bytes.capacity());
    assert_eq(bytes.get(0).unwrap(), 0u8);
    assert_eq(bytes.get(1).unwrap(), 1u8);
    assert_eq(bytes.get(2).unwrap(), 2u8);
}

#[test]
fn string_as_raw_slice() {
    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let raw_slice = bytes.as_raw_slice();
    let mut string = String::from(bytes);

    let string_slice = string.as_raw_slice();
    assert_eq(string_slice.number_of_bytes(), raw_slice.number_of_bytes());
}

#[test]
fn string_from_raw_slice() {
    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let raw_slice = bytes.as_raw_slice();
    let mut string_from_slice = String::from(raw_slice);
    let bytes = string_from_slice.as_bytes();
    assert_eq(bytes.len(), 5);
    assert_eq(bytes.get(0).unwrap(), 0u8);
    assert_eq(bytes.get(1).unwrap(), 1u8);
    assert_eq(bytes.get(2).unwrap(), 2u8);
}

#[test]
fn string_from_moved_raw_slice() {
    const LEN_IN_BYTES = 5;

    let content_ptr = std::alloc::alloc_bytes(LEN_IN_BYTES);
    __addr_of([0u8, 1u8, 2u8, 3u8, 4u8]).copy_bytes_to(content_ptr, LEN_IN_BYTES);

    let slice = asm(ptr: (content_ptr, LEN_IN_BYTES)) {
        ptr: raw_slice
    };

    let string = String::from_moved_raw_slice(slice);

    assert(string.ptr() == slice.ptr()); // String takes ownership of the slice
    assert_eq(string.len(), slice.number_of_bytes());
    assert_eq(string.len(), LEN_IN_BYTES);
    assert_eq(string.capacity(), LEN_IN_BYTES);

    let bytes = string.as_bytes();
    assert_eq(bytes.get(0).unwrap(), 0u8);
    assert_eq(bytes.get(1).unwrap(), 1u8);
    assert_eq(bytes.get(2).unwrap(), 2u8);
    assert_eq(bytes.get(3).unwrap(), 3u8);
    assert_eq(bytes.get(4).unwrap(), 4u8);
}

#[test]
fn string_into_raw_slice() {
    // Glob operator needed for From<String> for raw_slice
    use std::string::*;

    let mut string = String::new();

    let raw_slice: raw_slice = string.into();
    assert_eq(raw_slice.number_of_bytes(), 0);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    let raw_slice = string.as_raw_slice();
    assert_eq(raw_slice.number_of_bytes(), 1);
    assert_eq(raw_slice.ptr().read_byte(), 0u8);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.push(1u8);
    let string = String::from_ascii(bytes);
    let mut raw_slice = string.as_raw_slice();
    assert_eq(raw_slice.number_of_bytes(), 2);
    assert_eq(raw_slice.ptr().add_uint_offset(1).read_byte(), 1u8);

    let mut raw_slice = string.as_raw_slice();
    assert_eq(raw_slice.number_of_bytes(), 2);
    assert_eq(raw_slice.ptr().read_byte(), 0u8);
}

#[test]
fn string_raw_slice_into() {
    // Glob operator needed for From<String> for raw_slice
    use std::string::*;

    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let raw_slice = bytes.as_raw_slice();
    let mut string = String::from(bytes);

    let string_slice: raw_slice = string.into();
    assert_eq(string_slice.number_of_bytes(), raw_slice.number_of_bytes());
}

#[test]
fn string_raw_slice_from() {
    // Glob operator needed for From<String> for raw_slice
    use std::string::*;

    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let raw_slice = bytes.as_raw_slice();
    let mut string: String = String::from(bytes);

    let string_slice = raw_slice::from(string);
    assert_eq(string_slice.number_of_bytes(), raw_slice.number_of_bytes());
}

#[test]
fn string_test_equal() {
    let string1 = String::from_ascii_str("fuel");
    let string2 = String::from_ascii_str("fuel");
    let string3 = String::from_ascii_str("blazingly fast");

    assert_eq(string1, string2);
    assert_ne(string1, string3);
}

#[test]
fn string_test_hash() {
    use std::hash::{Hash, sha256};

    let mut bytes = Bytes::new();
    bytes.push(0u8);

    let string = String::from(bytes);

    assert_eq(sha256(string), sha256(bytes));
}

#[test]
fn string_test_abi_encoding() {
    let string = String::from_ascii_str("fuel");

    let buffer = Buffer::new();
    let encoded_string = string.abi_encode(buffer);

    let encoded_raw_slice = encoded_string.as_raw_slice();
    let mut buffer_reader = BufferReader::from_parts(encoded_raw_slice.ptr(), encoded_raw_slice.number_of_bytes());

    let decoded_string = String::abi_decode(buffer_reader);

    assert_eq(string, decoded_string);
}

#[test]
fn string_clone() {
    let string = String::from_ascii_str("fuel");

    let cloned_string = string.clone();

    assert(cloned_string.ptr() != string.ptr());
    assert_eq(cloned_string.len(), string.len());
    assert_eq(cloned_string.as_bytes().len(), string.as_bytes().len());
    assert_eq(
        cloned_string
            .as_bytes()
            .get(0)
            .unwrap(),
        string
            .as_bytes()
            .get(0)
            .unwrap(),
    );
    assert_eq(
        cloned_string
            .as_bytes()
            .get(1)
            .unwrap(),
        string
            .as_bytes()
            .get(1)
            .unwrap(),
    );
    assert_eq(
        cloned_string
            .as_bytes()
            .get(2)
            .unwrap(),
        string
            .as_bytes()
            .get(2)
            .unwrap(),
    );
}

#[test]
fn string_empty_as_str() {
    let string = String::from_ascii_str("");
    assert_eq(string.as_str(), "");
}

#[test]
fn string_as_str() {
    let string = String::from_ascii_str("Fuel");
    assert_eq(string.as_str(), "Fuel");
}
