library;

use ::bytes::Bytes;
use ::convert::From;
use ::option::Option;
use ::assert::assert;

/// A UTF-8 encoded growable string
///
/// WARNING: As this type is meant to be forward compatible with UTF-8, do *not*
/// add any mutation functionality or unicode input of any kind until `char` is
/// implemented, codepoints are *not* guaranteed to fall on byte boundaries
pub struct String {
    bytes: Bytes,
}

impl String {
    /// Returns `Bytes` giving a UTF-8 representation of the string.
    pub fn as_bytes(self) -> Bytes {
        self.bytes
    }

    /// Gets the amount of memory on the heap allocated to the `String`.
    pub fn capacity(self) -> u64 {
        self.bytes.capacity()
    }

    /// Truncates this `String` to a length of zero, clearing all content.
    pub fn clear(ref mut self) {
        self.bytes.clear()
    }

    /// Converts a vector of ASCII encoded bytes to a `String`.
    ///
    /// Each byte represents a single character, this supports ASCII but it does **not** support Unicode.
    ///
    /// # Arguments
    ///
    /// * `bytes` - ASCII bytes which will be converted into a `String`.
    pub fn from_ascii(bytes: Bytes) -> Self {
        Self {
            bytes,
        }
    }

    /// Returns `true` if the string is empty (contains no bytes).
    pub fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    /// Constructs a new instance of the `String` type.
    pub fn new() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Constructs a new instance of the `String` type with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The specified amount of memory on the heap to be allocated for the `String`.
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            bytes: Bytes::with_capacity(capacity),
        }
    }
}

impl From<Bytes> for String {
    fn from(b: Bytes) -> Self {
        let mut string = Self::new();
        string.bytes = b;
        string
    }

    fn into(self) -> Bytes {
        self.bytes
    }
}

impl AsRawSlice for String {
    /// Returns a raw slice to all of the elements in the string.
    fn as_raw_slice(self) -> raw_slice {
        asm(ptr: (self.bytes.buf.ptr(), self.bytes.len)) { ptr: raw_slice }
    }
}

impl From<raw_slice> for String {
    fn from(slice: raw_slice) -> String {
        Self {
            bytes: Bytes::from(slice),
        }
    }

    fn into(self) -> raw_slice {
        asm(ptr: (self.bytes.buf.ptr(), self.bytes.len)) { ptr: raw_slice }
    }
}

// Tests
//

#[test]
fn string_test_as_bytes() {
    let mut string = String::new();

    let bytes = string.as_bytes();
    assert(bytes.len() == 0);
    assert(bytes.capacity() == string.capacity());

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);

    let bytes = string.as_bytes();
    assert(bytes.len() == 1);
    assert(bytes.capacity() == string.capacity());
}

#[test]
fn string_test_capacity() {
    let mut string = String::new();

    assert(string.capacity() == 0);

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    assert(string.capacity() == 1);
}

#[test]
fn string_test_clear() {
    let mut string = String::new();

    assert(string.is_empty());

    string.clear();
    assert(string.is_empty());

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let mut string = String::from_ascii(bytes);
    assert(!string.is_empty());

    string.clear();
    assert(string.is_empty());
}

#[test]
fn string_test_from() {
    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let mut string_from_bytes = String::from(bytes);
    let bytes = string_from_bytes.as_bytes();
    assert(bytes.len() == 5);
    assert(bytes.capacity() == string_from_bytes.capacity());
    assert(bytes.get(0).unwrap() == 0u8);
    assert(bytes.get(1).unwrap() == 1u8);
    assert(bytes.get(2).unwrap() == 2u8);
}

#[test]
fn string_test_from_raw_slice() {
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
fn string_test_from_ascii() {
    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let mut string_from_ascii = String::from_ascii(bytes);
    assert(bytes.capacity() == string_from_ascii.capacity());
    assert(bytes.get(0).unwrap() == 0u8);
    assert(bytes.get(1).unwrap() == 1u8);
    assert(bytes.get(2).unwrap() == 2u8);
}

#[test]
fn string_test_into_bytes() {
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
fn string_test_into_raw_slice() {
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
fn string_test_is_empty() {
    let mut string = String::new();

    assert(string.is_empty());

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    let string = String::from_ascii(bytes);
    assert(!string.is_empty());

    let mut bytes = Bytes::new();
    bytes.push(0u8);
    bytes.push(1u8);
    let mut string = String::from_ascii(bytes);
    assert(!string.is_empty());

    string.clear();
    assert(string.is_empty());
}

#[test]
fn string_test_new() {
    let mut string = String::new();

    assert(string.is_empty());
    assert(string.capacity() == 0);
}

#[test]
fn string_test_with_capacity() {
    let mut iterator = 0;

    while iterator < 16 {
        let mut string = String::with_capacity(iterator);
        assert(string.capacity() == iterator);
        iterator += 1;
    }

    let mut string = String::with_capacity(0);
    assert(string.capacity() == 0);

    string.clear();
    assert(string.capacity() == 0);
    let mut string = String::with_capacity(4);

    assert(string.capacity() == 4);
}
