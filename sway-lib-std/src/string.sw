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
