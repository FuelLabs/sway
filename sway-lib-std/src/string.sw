//! A UTF-8 encoded growable string.
library;

use ::assert::assert;
use ::bytes::*;
use ::convert::*;
use ::hash::{Hash, Hasher};
use ::option::Option;

/// A UTF-8 encoded growable string.
///
/// # Additional Information
///
/// WARNING: As this type is meant to be forward compatible with UTF-8, do *not*
/// add any mutation functionality or unicode input of any kind until `char` is
/// implemented, codepoints are *not* guaranteed to fall on byte boundaries
pub struct String {
    /// The bytes representing the characters of the string.
    bytes: Bytes,
}

impl String {
    /// Returns `Bytes` giving a UTF-8 representation of the string.
    ///
    /// # Returns
    ///
    /// * [Bytes] - A UTF-8 representation of the string.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let mut string = String::new();
    ///     string.push(0u8);
    ///     let bytes = string.as_bytes();
    ///     assert(bytes.len() == 1);
    ///     assert(bytes.get(0).unwrap() == 0u8);
    /// }
    /// ```
    pub fn as_bytes(self) -> Bytes {
        self.bytes
    }

    /// Gets the amount of memory on the heap allocated to the `String`.
    ///
    /// # Returns
    ///
    /// * `u64` - The number of characters the `String` can hold without reallocating.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let mut string = String::new();
    ///     assert(string.capacity() == 0);
    ///     string.push(0u8);
    ///     assert(string.capacity() == 1);
    ///     string.push(1u8);
    ///     assert(string.capacity() == 2);
    /// }
    /// ```
    pub fn capacity(self) -> u64 {
        self.bytes.capacity()
    }

    /// Truncates this `String` to a length of zero, clearing all content.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let mut string = String::new();
    ///     string.push(0u8);
    ///     assert(!string.is_empty());
    ///     string.clear();
    ///     assert(string.is_empty());
    /// }
    /// ```
    pub fn clear(ref mut self) {
        self.bytes.clear()
    }

    /// Converts a vector of ASCII encoded bytes to a `String`.
    ///
    /// # Additional Information
    ///
    /// Each byte represents a single character, this supports ASCII but it does **not** support Unicode.
    ///
    /// # Arguments
    ///
    /// * `bytes` - ASCII bytes which will be converted into a `String`.
    ///
    /// # Returns
    ///
    /// * [String] - A `String` containing the ASCII encoded bytes.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(0u8);
    ///     bytes.push(1u8);
    ///     let string = String::from_ascii(bytes);
    /// }
    /// ```
    pub fn from_ascii(bytes: Bytes) -> Self {
        Self { bytes }
    }

    /// Converts a string slice containing ASCII encoded bytes to a `String`
    ///
    /// # Arguments
    ///
    /// * `s` - A string slice containing ASCII encoded bytes.
    ///
    /// # Returns
    ///
    /// * [String] - A `String` containing the ASCII encoded bytes.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("ABCDEF");
    /// }
    /// ```
    pub fn from_ascii_str(s: str) -> Self {
        let str_size = s.len();
        let str_ptr = s.as_ptr();

        Self {
            bytes: Bytes::from(raw_slice::from_parts::<u8>(str_ptr, str_size)),
        }
    }

    /// Returns a `bool` indicating whether the `String` is empty.
    ///
    /// # Returns
    ///
    /// * [bool] - `true` if the `String` is empty, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let mut string = String::new();
    ///     assert(string.is_empty());
    ///     string.push(0u8);
    ///     assert(!string.is_empty());
    /// }
    /// ```
    pub fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    /// Constructs a new instance of the `String` type.
    ///
    /// # Returns
    ///
    /// * [String] - A new empty instance of the `String` type.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let string = String::new();
    ///     string.push(0u8);
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Constructs a new instance of the `String` type with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity`: [u64] - The specified amount of bytes on the heap to be allocated for the `String`.
    ///
    /// # Returns
    ///
    /// * [String] - A new empty instance of the `String` type with the specified capacity.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let string = String::with_capacity(1);
    ///     string.push(0u8); // This will not reallocate
    ///     string.push(1u8); // This will reallocate
    /// }
    /// ```
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            bytes: Bytes::with_capacity(capacity),
        }
    }

    /// Gets the pointer of the allocation.
    ///
    /// # Returns
    ///
    /// [raw_ptr] - The location in memory that the allocated string lives.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let string = String::new();
    ///     assert(!string.ptr().is_null());
    /// }
    /// ```
    pub fn ptr(self) -> raw_ptr {
        self.bytes.ptr()
    }
}

impl From<Bytes> for String {
    fn from(b: Bytes) -> Self {
        Self { bytes: b }
    }
}

impl From<String> for Bytes {
    fn from(s: String) -> Bytes {
        s.as_bytes()
    }
}

impl AsRawSlice for String {
    /// Returns a raw slice to all of the elements in the string.
    fn as_raw_slice(self) -> raw_slice {
        self.bytes.as_raw_slice()
    }
}

impl From<raw_slice> for String {
    /// Converts a `raw_slice` to a `String`.
    ///
    /// # Arguments
    ///
    /// * `slice`: [raw_slice] - The `raw_slice` to convert to a `String`.
    ///
    /// # Returns
    ///
    /// * [String] - The newly created `String`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{alloc::alloc, string::*};
    ///
    /// fn foo() {
    ///     let ptr = alloc::<u64>(1);
    ///     let slice = raw_slice::from_parts::<u64>(ptr, 1);
    ///     let string: String = String::from(slice);
    /// }
    /// ```
    fn from(slice: raw_slice) -> Self {
        Self {
            bytes: Bytes::from(slice),
        }
    }
}

impl From<String> for raw_slice {
    /// Converts a `String` to a `raw_slice`.
    ///
    /// # Additional Information
    ///
    /// **NOTE:** To import, use the glob operator i.e. `use std::string::*;`
    ///
    /// # Arguments
    ///
    /// * `s`: [String] - The `String` to convert to a `raw_slice`.
    ///
    /// # Returns
    ///
    /// * [raw_slice] - The newly created `raw_slice`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::*;
    ///
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel");
    ///     let string_slice: raw_slice = string.into();
    /// }
    /// ```
    fn from(s: String) -> raw_slice {
        raw_slice::from(s.as_bytes())
    }
}

impl Eq for String {
    fn eq(self, other: Self) -> bool {
        self.bytes == other.as_bytes()
    }
}

impl Hash for String {
    fn hash(self, ref mut state: Hasher) {
        state.write(self.bytes);
    }
}

impl AbiEncode for String {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        // Encode the length
        let mut buffer = self.bytes.len().abi_encode(buffer);

        // Encode each byte of the string
        let mut i = 0;
        while i < self.bytes.len() {
            let item = self.bytes.get(i).unwrap();
            buffer = item.abi_encode(buffer);
            i += 1;
        }

        buffer
    }
}

impl AbiDecode for String {
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        // Get length and string data
        let len = u64::abi_decode(buffer);
        let data = buffer.read_bytes(len);
        // Create string from the ptr and len as parts of a raw_slice
        String {
            bytes: Bytes::from(raw_slice::from_parts::<u8>(data.ptr(), len)),
        }
    }
}

#[test]
fn ok_string_buffer_ownership() {
    use ::option::Option::Some;

    let mut string_slice = "hi";
    let mut string = String::from_ascii_str(string_slice);

    // change first char to 'H'
    let mut bytes = string.as_bytes();
    bytes.set(0, 72);

    // Check the string changed, but not the original slice
    assert(string.as_bytes().get(0) == Some(72));
    assert(string_slice == "hi");

    // encoded bytes should be <length> Hi
    let encoded_bytes = encode(string);
    let string = abi_decode::<String>(encoded_bytes);

    // change first char to 'P'
    string.as_bytes().set(0, 80);

    // Check decoded string is "Pi"
    assert(string.as_bytes().get(0) == Some(80));

    // Check original string slice has not changed
    assert(string_slice == "hi");

    // Check encoded bytes has not changed
    let mut bytes = abi_decode::<Bytes>(encoded_bytes);
    assert(bytes.get(0) == Some(72));
}
