library;

use std::{bytes::Bytes, string::String};

#[test]
fn string_as_bytes() {
    let mut string = String::new();

    let bytes = string.as_bytes();
    assert(bytes.len() == 0);
    assert(bytes.capacity() == string.capacity());
    assert(bytes.len() == string.len());

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);

    let bytes = string.as_bytes();
    assert(bytes.len() == 1);
    assert(bytes.capacity() == string.capacity());
    assert(bytes.len() == string.len());
}

#[test]
fn string_capacity() {
    let mut string = String::new();
    assert(string.capacity() == 0);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    assert(string.capacity() == 1);
}

#[test]
fn string_len() {
    let mut string = String::new();
    assert(string.len() == 0);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    assert(string.len() == 1);

    let mut string = String::from_ascii_str("ABCDEF");
    assert(string.len() == 6);
}

#[test]
fn string_clear() {
    // Clear non-empty
    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let mut string = String::from_ascii(bytes);
    assert(!string.is_empty());
    assert(string.len() == 1);

    string.clear();
    assert(string.is_empty());
    assert(string.len() == 0);
}

#[test]
fn string_clear_empty() {
    let mut string = String::new();

    assert(string.is_empty());
    assert(string.len() == 0);
    string.clear();
    assert(string.is_empty());
    assert(string.len() == 0);
}

#[test]
fn string_from_ascii() {
    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let mut string_from_ascii = String::from_ascii(bytes);
    assert(bytes.len() == string_from_ascii.capacity());
    assert(bytes.len() == string_from_ascii.len());

    let bytes = string_from_ascii.as_bytes();
    assert(bytes.get(0).unwrap() == 0u8);
    assert(bytes.get(1).unwrap() == 1u8);
    assert(bytes.get(2).unwrap() == 2u8);
    assert(bytes.get(3).unwrap() == 3u8);
    assert(bytes.get(4).unwrap() == 4u8);
    assert(bytes.get(5) == None);
}

#[test]
fn string_from_ascii_str() {
    let mut string_from_ascii = String::from_ascii_str("ABCDEF");
    assert(string_from_ascii.capacity() == 6);
    assert(string_from_ascii.len() == 6);

    let bytes = string_from_ascii.as_bytes();
    assert(bytes.get(0).unwrap() == 65u8);
    assert(bytes.get(1).unwrap() == 66u8);
    assert(bytes.get(2).unwrap() == 67u8);
    assert(bytes.get(3).unwrap() == 68u8);
    assert(bytes.get(4).unwrap() == 69u8);
    assert(bytes.get(5).unwrap() == 70u8);
    assert(bytes.get(6).is_none());
}

#[test]
fn string_is_empty() {
    let mut string = String::new();
    assert(string.is_empty());
    assert(string.len() == 0);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    assert(!string.is_empty());
    assert(string.len() == 1);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.push(1u8);
    let mut string = String::from_ascii(bytes);
    assert(!string.is_empty());
    assert(string.len() == 2);

    string.clear();
    assert(string.is_empty());
    assert(string.len() == 0);
}

#[test]
fn string_new() {
    let mut string = String::new();

    assert(string.is_empty());
    assert(string.capacity() == 0);
    assert(string.len() == 0);
}

#[test]
fn string_with_capacity() {
    let mut iterator = 0;

    while iterator < 16 {
        let mut string = String::with_capacity(iterator);
        assert(string.capacity() == iterator);
        iterator += 1;
    }

    let mut string = String::with_capacity(0);
    assert(string.capacity() == 0);
    assert(string.len() == 0);

    string.clear();
    assert(string.capacity() == 0);
    assert(string.len() == 0);

    let mut string = String::with_capacity(4);
    assert(string.capacity() == 4);
    assert(string.len() == 0);
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

    let mut string_from_ascii = String::from_ascii(bytes);
    assert(!string_from_ascii.ptr().is_null());
    assert(string_from_ascii.ptr() != bytes.ptr());
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
    assert(bytes.len() == 5);
    assert(string_from_bytes.len() == 5);
    assert(bytes.capacity() == string_from_bytes.capacity());
    assert(bytes.get(0).unwrap() == 0u8);
    assert(bytes.get(1).unwrap() == 1u8);
    assert(bytes.get(2).unwrap() == 2u8);
}

#[test]
fn string_into_bytes() {
    let mut string = String::new();

    let bytes: Bytes = string.into();
    assert(bytes.len() == 0);
    assert(bytes.capacity() == string.capacity());

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    let bytes: Bytes = string.into();
    assert(bytes.len() == 1);
    assert(bytes.capacity() == string.capacity());
    assert(bytes.get(0).unwrap() == 0u8);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.push(1u8);
    let string = String::from_ascii(bytes);
    let mut bytes: Bytes = string.into();
    assert(bytes.len() == 2);
    assert(bytes.capacity() == string.capacity());
    assert(bytes.get(1).unwrap() == 1u8);
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
    assert(bytes.len() == 5);
    assert(bytes.capacity() == string_from_bytes.capacity());
    assert(bytes.get(0).unwrap() == 0u8);
    assert(bytes.get(1).unwrap() == 1u8);
    assert(bytes.get(2).unwrap() == 2u8);
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
    assert(bytes.len() == 5);
    assert(bytes.capacity() == string_from_bytes.capacity());
    assert(bytes.get(0).unwrap() == 0u8);
    assert(bytes.get(1).unwrap() == 1u8);
    assert(bytes.get(2).unwrap() == 2u8);
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
    assert(string_slice.number_of_bytes() == raw_slice.number_of_bytes());
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
    assert(bytes.len() == 5);
    assert(bytes.get(0).unwrap() == 0u8);
    assert(bytes.get(1).unwrap() == 1u8);
    assert(bytes.get(2).unwrap() == 2u8);
}

#[test]
fn string_into_raw_slice() {
    // Glob operator needed for From<String> for raw_slice
    use std::string::*;

    let mut string = String::new();

    let raw_slice: raw_slice = string.into();
    assert(raw_slice.number_of_bytes() == 0);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    let raw_slice = string.as_raw_slice();
    assert(raw_slice.number_of_bytes() == 1);
    assert(raw_slice.ptr().read_byte() == 0u8);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.push(1u8);
    let string = String::from_ascii(bytes);
    let mut raw_slice = string.as_raw_slice();
    assert(raw_slice.number_of_bytes() == 2);
    assert(raw_slice.ptr().add_uint_offset(1).read_byte() == 1u8);

    let mut raw_slice = string.as_raw_slice();
    assert(raw_slice.number_of_bytes() == 2);
    assert(raw_slice.ptr().read_byte() == 0u8);
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
    assert(string_slice.number_of_bytes() == raw_slice.number_of_bytes());
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
    assert(string_slice.number_of_bytes() == raw_slice.number_of_bytes());
}

#[test]
fn string_test_equal() {
    let string1 = String::from_ascii_str("fuel");
    let string2 = String::from_ascii_str("fuel");
    let string3 = String::from_ascii_str("blazingly fast");

    assert(string1 == string2);
    assert(string1 != string3);
}

#[test]
fn string_test_hash() {
    use std::hash::{Hash, sha256};

    let mut bytes = Bytes::new();
    bytes.push(0u8);

    let string = String::from(bytes);

    assert(sha256(string) == sha256(bytes));
}

#[test]
fn string_test_abi_encoding() {
    let string = String::from_ascii_str("fuel");

    let buffer = Buffer::new();
    let encoded_string = string.abi_encode(buffer);

    let encoded_raw_slice = encoded_string.as_raw_slice();
    let mut buffer_reader = BufferReader::from_parts(encoded_raw_slice.ptr(), encoded_raw_slice.number_of_bytes());

    let decoded_string = String::abi_decode(buffer_reader);

    assert(string == decoded_string);
}

#[test]
fn string_clone() {
    let string = String::from_ascii_str("fuel");

    let cloned_string = string.clone();

    assert(cloned_string.ptr() != string.ptr());
    assert(cloned_string.len() == string.len());
    assert(cloned_string.as_bytes().len() == string.as_bytes().len());
    assert(cloned_string.as_bytes().get(0).unwrap() == string.as_bytes().get(0).unwrap());
    assert(cloned_string.as_bytes().get(1).unwrap() == string.as_bytes().get(1).unwrap());
    assert(cloned_string.as_bytes().get(2).unwrap() == string.as_bytes().get(2).unwrap());
}
