library;

use ::bytes::Bytes;
use ::convert::From;
use ::vec::Vec;
use ::option::Option;
use ::assert::assert;

pub struct String {
    bytes: Bytes,
}

impl String {
    /// Returns a `Vec<u8>` of the bytes stored for the `String`.
    pub fn as_vec(self) -> Vec<u8> {
        self.bytes.into_vec_u8()
    }

    /// Gets the amount of memory on the heap allocated to the `String`.
    pub fn capacity(self) -> u64 {
        self.bytes.capacity()
    }

    /// Truncates this `String` to a length of zero, removing all contents.
    pub fn clear(ref mut self) {
        self.bytes.clear()
    }

    /// Converts a vector of bytes to a `String`.
    ///
    /// # Arguments
    ///
    /// * `bytes` - The vector of `u8` bytes which will be converted into a `String`.
    pub fn from_utf8(bytes: Vec<u8>) -> Self {
        let mut bytes = bytes;
        Self {
            bytes: Bytes::from_vec_u8(bytes),
        }
    }

    /// Inserts a byte at the index within the `String`.
    ///
    /// # Arguments
    ///
    /// * `byte` - The element which will be added to the `String`.
    /// * `index` - The position in the `String` where the byte will be inserted.
    pub fn insert(ref mut self, byte: u8, index: u64) {
        self.bytes.insert(index, byte);
    }

    /// Returns `true` if the vector contains no bytes.
    pub fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    /// Returns the number of bytes in the `String`, also referred to
    /// as its 'length'.
    pub fn len(self) -> u64 {
        self.bytes.len()
    }

    /// Constructs a new instance of the `String` type.
    pub fn new() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Returns the byte at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The position of the byte that will be returned.
    pub fn nth(self, index: u64) -> Option<u8> {
        self.bytes.get(index)
    }

    /// Removes the last byte from the `String` buffer and returns it.
    pub fn pop(ref mut self) -> Option<u8> {
        self.bytes.pop()
    }

    /// Appends a byte to the end of the `String`.
    ///
    /// # Arguments
    ///
    /// * `byte` - The element to be appended to the end of the `String`.
    pub fn push(ref mut self, byte: u8) {
        self.bytes.push(byte);
    }

    /// Removes and returns the byte at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The position of the byte that will be removed.
    pub fn remove(ref mut self, index: u64) -> u8 {
        self.bytes.remove(index)
    }

    /// Updates a byte at position `index` with a new byte `value`.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the byte to be set.
    /// * `byte` - The value of the byte to be set.
    pub fn set(ref mut self, index: u64, byte: u8) {
        self.bytes.set(index, byte);
    }

    /// Swaps two bytes.
    ///
    /// # Arguments
    ///
    /// * `byte1_index` - The index of the first byte.
    /// * `byte2_index` - The index of the second byte.
    pub fn swap(ref mut self, byte1_index: u64, byte2_index: u64) {
        self.bytes.swap(byte1_index, byte2_index);
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

impl String {
    /// Moves all elements of the `other` String into `self`, leaving `other` empty.
    ///
    /// # Arguments
    ///
    /// * `other` - The String to join to self.
    pub fn append(ref mut self, ref mut other: self) {
        self.bytes.append(other.bytes);
    }

    /// Divides one Bytes into two at an index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to split the original String at.
    pub fn split_at(ref mut self, index: u64) -> (Self, Self) {
        let (bytes1, bytes2) = self.bytes.split_at(index);
        (Self::from(bytes1), Self::from(bytes2))
    }
}

impl From<raw_slice> for String {
    fn from(slice: raw_slice) -> String {
        Self {
            bytes: Bytes::from_raw_slice(slice),
        }
    }

    fn into(self) -> raw_slice {
        asm(ptr: (self.bytes.buf.ptr(), self.bytes.len)) { ptr: raw_slice }
    }
}

// Tests
//

#[test()]
fn string_test_append() {
    let mut string1 = String::new();
    let mut string2 = String::new();

    string1.push(0u8);
    string1.push(1u8);
    string1.push(2u8);
    string2.push(3u8);
    string2.push(4u8);
    string2.push(5u8);

    assert(string2.len() == 3);
    assert(string1.len() == 3);

    string1.append(string2);

    assert(string2.len() == 0);
    assert(string1.len() == 6);
    assert(string1.nth(0).unwrap() == 0u8);
    assert(string1.nth(1).unwrap() == 1u8);
    assert(string1.nth(2).unwrap() == 2u8);
    assert(string1.nth(3).unwrap() == 3u8);
    assert(string1.nth(4).unwrap() == 4u8);
    assert(string1.nth(5).unwrap() == 5u8);
}

#[test()]
fn string_test_as_vec() {
    let mut string = String::new();

    let bytes = string.as_vec();
    assert(bytes.len() == string.len());
    assert(bytes.capacity() == string.capacity());

    string.push(0u8);
    let bytes = string.as_vec();
    assert(bytes.len() == string.len());
    assert(bytes.capacity() == string.capacity());
    assert(bytes.get(0).unwrap() == string.nth(0).unwrap());

    string.push(1u8);
    let mut bytes = string.as_vec();
    assert(bytes.len() == string.len());
    assert(bytes.capacity() == string.capacity());
    assert(bytes.get(1).unwrap() == string.nth(1).unwrap());
}

#[test()]
fn string_test_capacity() {
    let mut string = String::new();

    assert(string.capacity() == 0);

    string.push(0u8);
    assert(string.capacity() == 1);

    string.push(1u8);
    assert(string.capacity() == 2);

    string.push(2u8);
    assert(string.capacity() == 4);
    string.push(3u8);
    assert(string.capacity() == 4);

    string.push(4u8);
    assert(string.capacity() == 8);
    string.push(5u8);
    assert(string.capacity() == 8);
    string.push(6u8);
    string.push(7u8);
    assert(string.capacity() == 8);

    string.push(8u8);
    assert(string.capacity() == 16);

    string.clear();
    assert(string.capacity() == 0);

    string.push(0u8);
    assert(string.capacity() == 1);
}

#[test()]
fn string_test_clear() {
    let mut string = String::new();

    assert(string.is_empty());

    string.clear();
    assert(string.is_empty());

    string.push(0u8);
    assert(!string.is_empty());

    string.clear();
    assert(string.is_empty());

    string.push(0u8);
    string.push(1u8);
    string.push(2u8);
    string.push(3u8);
    string.push(4u8);
    string.push(5u8);
    string.push(6u8);
    string.push(7u8);
    string.push(8u8);
    assert(!string.is_empty());

    string.clear();
    assert(string.is_empty());

    string.clear();
    assert(string.is_empty());

    string.push(0u8);
    assert(!string.is_empty());

    string.clear();
    assert(string.is_empty());
}

#[test()]
fn string_test_from() {
    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let mut string_from_bytes = String::from(bytes);
    assert(bytes.len() == string_from_bytes.len());
    assert(bytes.capacity() == string_from_bytes.capacity());
    assert(bytes.get(0).unwrap() == string_from_bytes.nth(0).unwrap());
    assert(bytes.get(1).unwrap() == string_from_bytes.nth(1).unwrap());
    assert(bytes.get(2).unwrap() == string_from_bytes.nth(2).unwrap());
}

#[test()]
fn string_test_from_raw_slice() {
    let mut bytes = Bytes::new();

    bytes.push(0u8);
    bytes.push(1u8);
    bytes.push(2u8);
    bytes.push(3u8);
    bytes.push(4u8);

    let raw_slice = bytes.as_raw_slice();
    let mut string_from_slice = String::from(raw_slice);
    assert(bytes.len() == string_from_slice.len());
    assert(bytes.get(0).unwrap() == string_from_slice.nth(0).unwrap());
    assert(bytes.get(1).unwrap() == string_from_slice.nth(1).unwrap());
    assert(bytes.get(2).unwrap() == string_from_slice.nth(2).unwrap());
}

#[test()]
fn string_test_from_utf8() {
    let mut vec: Vec<u8> = Vec::new();

    vec.push(0u8);
    vec.push(1u8);
    vec.push(2u8);
    vec.push(3u8);
    vec.push(4u8);

    let mut string_from_uft8 = String::from_utf8(vec);
    assert(vec.len() == string_from_uft8.len());
    assert(vec.capacity() == string_from_uft8.capacity());
    assert(vec.get(0).unwrap() == string_from_uft8.nth(0).unwrap());
    assert(vec.get(1).unwrap() == string_from_uft8.nth(1).unwrap());
    assert(vec.get(2).unwrap() == string_from_uft8.nth(2).unwrap());
}

#[test()]
fn string_test_insert() {
    let mut string = String::new();

    assert(string.len() == 0);

    string.insert(0u8, 0);
    assert(string.len() == 1);
    assert(string.nth(0).unwrap() == 0u8);

    string.push(1u8);
    string.push(2u8);
    string.insert(3u8, 0);
    assert(string.len() == 4);
    assert(string.nth(0).unwrap() == 3u8);

    string.insert(4u8, 1);
    assert(string.nth(1).unwrap() == 4u8);

    string.insert(5u8, string.len() - 1);
    assert(string.nth(string.len() - 2).unwrap() == 5u8);
}

#[test()]
fn string_test_into_bytes() {
    let mut string = String::new();

    let bytes: Bytes = string.into();
    assert(bytes.len() == string.len());
    assert(bytes.capacity() == string.capacity());

    string.push(0u8);
    let bytes: Bytes = string.into();
    assert(bytes.len() == string.len());
    assert(bytes.capacity() == string.capacity());
    assert(bytes.get(0).unwrap() == string.nth(0).unwrap());

    string.push(1u8);
    let mut bytes: Bytes = string.into();
    assert(bytes.len() == string.len());
    assert(bytes.capacity() == string.capacity());
    assert(bytes.get(1).unwrap() == string.nth(1).unwrap());

    let result_string = string.pop().unwrap();
    let result_bytes = bytes.pop().unwrap();
    assert(result_bytes == result_string);
    assert(bytes.len() == string.len());
    assert(bytes.capacity() == string.capacity());
    assert(bytes.get(0).unwrap() == string.nth(0).unwrap());
}

#[test()]
fn string_test_into_raw_slice() {
    let mut string = String::new();

    let raw_slice: raw_slice = string.into();
    assert(raw_slice.number_of_bytes() == string.len());

    string.push(0u8);
    let raw_slice = string.as_raw_slice();
    assert(raw_slice.number_of_bytes() == string.len());
    assert(raw_slice.ptr().read_byte() == string.nth(0).unwrap());

    string.push(1u8);
    let mut raw_slice = string.as_raw_slice();
    assert(raw_slice.number_of_bytes() == string.len());
    assert(raw_slice.ptr().add_uint_offset(1).read_byte() == string.nth(1).unwrap());

    let mut raw_slice = string.as_raw_slice();
    assert(raw_slice.number_of_bytes() == string.len());
    assert(raw_slice.ptr().read_byte() == string.nth(0).unwrap());
}

#[test()]
fn string_test_is_empty() {
    let mut string = String::new();

    assert(string.is_empty());

    string.push(0u8);
    assert(!string.is_empty());

    string.push(1u8);
    assert(!string.is_empty());

    string.clear();
    assert(string.is_empty());

    string.push(0u8);
    assert(!string.is_empty());

    string.push(1u8);
    assert(!string.is_empty());

    let _result = string.pop();
    assert(!string.is_empty());

    let _result = string.pop();
    assert(string.is_empty());
}

#[test()]
fn string_test_len() {
    let mut string = String::new();

    assert(string.len() == 0);

    string.push(0u8);
    assert(string.len() == 1);

    string.push(1u8);
    assert(string.len() == 2);

    string.push(2u8);
    assert(string.len() == 3);

    string.push(3u8);
    assert(string.len() == 4);

    string.push(4u8);
    assert(string.len() == 5);

    string.push(5u8);
    assert(string.len() == 6);
    string.push(6u8);
    assert(string.len() == 7);

    string.push(7u8);
    assert(string.len() == 8);

    string.push(8u8);
    assert(string.len() == 9);
    let _result = string.pop();
    assert(string.len() == 8);

    string.clear();
    assert(string.len() == 0);
}

#[test()]
fn string_test_new() {
    let mut string = String::new();

    assert(string.len() == 0);
    assert(string.is_empty());
    assert(string.capacity() == 0);
}

#[test()]
fn string_test_nth() {
    let mut string = String::new();

    string.push(0u8);
    assert(string.nth(0).unwrap() == 0u8);

    string.push(1u8);
    assert(string.nth(0).unwrap() == 0u8);
    assert(string.nth(1).unwrap() == 1u8);

    string.push(2u8);
    assert(string.nth(0).unwrap() == 0u8);
    assert(string.nth(1).unwrap() == 1u8);
    assert(string.nth(2).unwrap() == 2u8);

    string.push(3u8);
    assert(string.nth(0).unwrap() == 0u8);
    assert(string.nth(1).unwrap() == 1u8);
    assert(string.nth(2).unwrap() == 2u8);
    assert(string.nth(3).unwrap() == 3u8);

    string.push(4u8);
    assert(string.nth(0).unwrap() == 0u8);
    assert(string.nth(1).unwrap() == 1u8);
    assert(string.nth(2).unwrap() == 2u8);
    assert(string.nth(3).unwrap() == 3u8);
    assert(string.nth(4).unwrap() == 4u8);

    string.clear();
    string.push(5u8);
    string.push(6u8);
    assert(string.nth(0).unwrap() == 5u8);
    assert(string.nth(1).unwrap() == 6u8);

    assert(string.nth(2).is_none());
}

#[test()]
fn string_test_pop() {
    let mut string = String::new();

    string.push(0u8);
    string.push(1u8);
    string.push(2u8);
    string.push(3u8);
    string.push(4u8);

    assert(string.len() == 5);
    assert(string.pop().unwrap() == 4u8);

    assert(string.len() == 4);
    assert(string.pop().unwrap() == 3u8);

    assert(string.len() == 3);
    assert(string.pop().unwrap() == 2u8);

    assert(string.len() == 2);
    assert(string.pop().unwrap() == 1u8);
    assert(string.len() == 1);
    assert(string.pop().unwrap() == 0u8);

    assert(string.len() == 0);
    assert(string.pop().is_none());
    string.push(5u8);
    assert(string.pop().unwrap() == 5u8);
}

#[test()]
fn string_test_push() {
    let mut string = String::new();

    assert(string.len() == 0);
    assert(string.is_empty());
    assert(string.capacity() == 0);

    string.push(0u8);
    assert(string.nth(0).unwrap() == 0u8);
    assert(string.len() == 1);

    string.push(1u8);
    assert(string.nth(1).unwrap() == 1u8);
    assert(string.len() == 2);

    string.push(2u8);
    assert(string.nth(2).unwrap() == 2u8);
    assert(string.len() == 3);

    string.push(3u8);
    assert(string.nth(3).unwrap() == 3u8);
    assert(string.len() == 4);

    string.push(4u8);
    assert(string.nth(4).unwrap() == 4u8);
    assert(string.len() == 5);

    string.push(5u8);
    assert(string.nth(5).unwrap() == 5u8);
    assert(string.len() == 6);

    string.push(6u8);
    assert(string.nth(6).unwrap() == 6u8);
    assert(string.len() == 7);

    string.push(7u8);
    assert(string.nth(7).unwrap() == 7u8);
    assert(string.len() == 8);

    string.push(8u8);
    assert(string.nth(8).unwrap() == 8u8);
    assert(string.len() == 9);

    string.push(1u8);
    assert(string.nth(9).unwrap() == 1u8);
    assert(string.len() == 10);

    string.clear();
    assert(string.len() == 0);
    assert(string.is_empty());
    string.push(1u8);
    assert(string.nth(0).unwrap() == 1u8);
    assert(string.len() == 1);

    string.push(1u8);
    assert(string.nth(1).unwrap() == 1u8);
    assert(string.len() == 2);

    string.push(0u8);
    assert(string.nth(2).unwrap() == 0u8);
    assert(string.len() == 3);
}

#[test()]
fn string_test_set() {
    let mut string = String::new();

    string.push(0u8);
    string.push(1u8);
    string.push(2u8);

    assert(string.nth(0).unwrap() == 0u8);
    assert(string.nth(1).unwrap() == 1u8);
    assert(string.nth(2).unwrap() == 2u8);

    string.set(0, 3u8);
    assert(string.nth(0).unwrap() == 3u8);
    assert(string.len() == 3);

    string.set(1, 4u8);
    assert(string.nth(1).unwrap() == 4u8);
    assert(string.len() == 3);

    string.set(2, 5u8);
    assert(string.nth(2).unwrap() == 5u8);

    assert(string.len() == 3);
}

#[test()]
fn string_test_split_at() {
    let mut string1 = String::new();

    string1.push(0u8);
    string1.push(1u8);
    string1.push(2u8);
    string1.push(3u8);

    let (mut string2, mut string3) = string1.split_at(2);

    assert(string2.len() == 2);
    assert(string3.len() == 2);

    assert(string2.nth(0).unwrap() == 0u8);
    assert(string2.nth(1).unwrap() == 1u8);
    assert(string3.nth(0).unwrap() == 2u8);
    assert(string3.nth(1).unwrap() == 3u8);
}

#[test()]
fn string_test_swap() {
    let mut string = String::new();

    string.push(0u8);
    string.push(1u8);
    string.push(2u8);

    assert(string.nth(0).unwrap() == 0u8);
    assert(string.nth(1).unwrap() == 1u8);
    string.swap(0, 1);
    assert(string.nth(0).unwrap() == 1u8);
    assert(string.nth(1).unwrap() == 0u8);

    assert(string.nth(1).unwrap() == 0u8);
    assert(string.nth(2).unwrap() == 2u8);
    string.swap(1, 2);
    assert(string.nth(1).unwrap() == 2u8);
    assert(string.nth(2).unwrap() == 0u8);

    assert(string.nth(0).unwrap() == 1u8);
    assert(string.nth(2).unwrap() == 0u8);
    string.swap(0, 2);
    assert(string.nth(0).unwrap() == 0u8);
    assert(string.nth(2).unwrap() == 1u8);
}

#[test()]
fn string_test_remove() {
    let mut string = String::new();

    string.push(0u8);
    string.push(1u8);
    string.push(2u8);
    string.push(3u8);
    string.push(4u8);
    string.push(5u8);

    assert(string.len() == 6);

    assert(string.remove(0) == 0u8);
    assert(string.len() == 5);
    assert(string.remove(0) == 1u8);
    assert(string.len() == 4);

    assert(string.remove(1) == 3u8);
    assert(string.len() == 3);

    assert(string.remove(string.len() - 1) == 5u8);
    assert(string.len() == 2);

    assert(string.remove(1) == 4u8);
    assert(string.len() == 1);

    assert(string.remove(0) == 2u8);
    assert(string.len() == 0);

    string.push(6u8);
    assert(string.remove(0) == 6u8);
    assert(string.len() == 0);
}

#[test()]
fn string_test_with_capacity() {
    let mut iterator = 0;

    while iterator < 16 {
        let mut string = String::with_capacity(iterator);
        assert(string.capacity() == iterator);
        iterator += 1;
    }

    let mut string = String::with_capacity(0);
    assert(string.capacity() == 0);

    string.push(0u8);
    assert(string.capacity() == 1);

    string.push(1u8);
    assert(string.capacity() == 2);

    string.push(2u8);
    assert(string.capacity() == 4);

    string.clear();
    assert(string.capacity() == 0);
    let mut string = String::with_capacity(4);

    assert(string.capacity() == 4);

    string.push(0u8);
    assert(string.capacity() == 4);
    string.push(1u8);
    assert(string.capacity() == 4);

    string.push(2u8);
    assert(string.capacity() == 4);

    string.push(3u8);
    assert(string.capacity() == 4);

    string.push(4u8);
    assert(string.capacity() == 8);
}
