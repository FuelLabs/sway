//! A UTF-8 encoded growable string.
library;

use ::assert::assert_eq;
use ::bytes::*;
use ::convert::*;
use ::hash::{Hash, Hasher};
use ::option::Option;
use ::codec::*;
use ::debug::*;
use ::ops::*;
use ::raw_slice::AsRawSlice;
use ::clone::Clone;

/// A UTF-8 encoded growable string, that has ownership of its buffer.
///
/// # Additional Information
///
/// WARNING: As this type is meant to be forward compatible with UTF-8, do *not*
/// add any mutation functionality or unicode input of any kind until `char` is
/// implemented. Currently, codepoints are *not* guaranteed to fall on byte boundaries.
pub struct String {
    /// The bytes representing the characters of the string.
    bytes: Bytes,
}

impl String {
    /// Returns `Bytes` giving a UTF-8 representation of the string.
    ///
    /// # Additional Information
    ///
    /// The returned `Bytes` contains a copy of the underlying string bytes.
    /// To get string bytes without creating a copy of the underlying bytes,
    /// use `String::as_raw_slice`.
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
    ///     let string = String::from_ascii_str("Fuel");
    ///     let bytes = string.as_bytes();
    ///     assert_eq(bytes.len(), 4);
    ///     assert_eq(bytes.get(0).unwrap(), 70u8); // "F"
    ///     assert(bytes.ptr() != string.ptr()); // A copy is returned.
    /// }
    /// ```
    pub fn as_bytes(self) -> Bytes {
        self.bytes.clone()
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
    ///     let string = String::new();
    ///     assert_eq(string.capacity(), 0);
    ///     let mut string = String::from_ascii_str("Fuel");
    ///     assert_eq(string.capacity(), 4);
    ///     string.clear();
    ///     assert_eq(string.capacity(), 4); // Clearing does not change the capacity.
    /// }
    /// ```
    pub fn capacity(self) -> u64 {
        self.bytes.capacity()
    }

    /// Truncates this `String` to a length of zero, clearing all content.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the `String`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let mut string = String::from_ascii_str("Fuel");
    ///     assert(!string.is_empty());
    ///     assert_eq(string.capacity(), 4);
    ///     string.clear();
    ///     assert(string.is_empty());
    ///     assert_eq(string.capacity(), 4); // Clearing does not change the capacity.
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
    /// The content of `bytes` gets copied into the newly created `String`.
    /// To take the ownership of the `bytes` and move them into the newly
    /// created `String` without copying the content, use `String::from_moved_ascii`.
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
    ///     bytes.push(70u8); // "F"
    ///     bytes.push(117u8); // "u"
    ///     bytes.push(101u8); // "e"
    ///     bytes.push(108u8); // "l"
    ///     let string = String::from_ascii(bytes);
    ///     assert_eq(string.len(), 4);
    /// }
    /// ```
    pub fn from_ascii(bytes: Bytes) -> Self {
        Self {
            bytes: bytes.clone(),
        }
    }

    /// Converts a vector of ASCII encoded bytes to a `String`, taking the
    /// ownership of the `bytes`.
    ///
    /// # Additional Information
    ///
    /// Each byte represents a single character, this supports ASCII but it does **not** support Unicode.
    ///
    /// `bytes` **must not be used after the ownership is transferred to the
    /// newly created `String`**. Violating this restriction results in an undefined behavior.
    ///
    /// To convert the `bytes` to a `String` by copying its content, and without
    /// taking the ownership, use `String::from_ascii`.
    ///
    /// # Arguments
    ///
    /// * `bytes` - ASCII bytes which will be moved into a `String`.
    ///
    /// # Returns
    ///
    /// * [String] - A `String` containing the ASCII encoded bytes.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::{bytes::Bytes, string::String};
    ///
    /// fn foo() {
    ///     let mut bytes = Bytes::new();
    ///     bytes.push(70u8); // "F"
    ///     bytes.push(117u8); // "u"
    ///     bytes.push(101u8); // "e"
    ///     bytes.push(108u8); // "l"
    ///     let string = String::from_moved_ascii(bytes);
    ///
    ///     // ** `bytes` must not be used after this point. **
    ///
    ///     assert_eq(string.len(), 4);
    /// }
    /// ```
    pub fn from_moved_ascii(bytes: Bytes) -> Self {
        Self { bytes }
    }

    /// Converts a string slice containing ASCII encoded bytes to a `String`.
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
    ///     assert_eq(string.len(), 6);
    /// }
    /// ```
    pub fn from_ascii_str(s: str) -> Self {
        Self {
            bytes: Bytes::from(__transmute::<str, raw_slice>(s)),
        }
    }

    /// Converts a string array containing ASCII encoded bytes to a `String`.
    ///
    /// # Arguments
    ///
    /// * `s` - A string array containing ASCII encoded bytes.
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
    ///     let string = String::from_ascii_str_array(__to_str_array("ABCDEF"));
    ///     assert_eq(string.len(), 6);
    /// }
    /// ```
    pub fn from_ascii_str_array<const N: u64>(s: str[N]) -> Self {
        Self {
            bytes: Bytes::from(__transmute::<(raw_ptr, u64), raw_slice>((__addr_of(s), N))),
        }
    }

    /// Constructs a new `String` that takes the ownership of the `slice`.
    ///
    /// # Additional Information
    ///
    /// `slice` **must point to a heap-allocated memory** and, together with its
    /// owner, **must not be used after the ownership is transferred to the newly
    /// created `String`**. Violating these restrictions results in an undefined behavior.
    ///
    /// To create a new `String` from a `raw_slice` that copies the slice content
    /// and does not take the ownership, use `String::from(raw_slice)`.
    ///
    /// # Arguments
    ///
    /// * `slice`: [raw_slice] - The heap-allocated slice whose ownership is transferred to the `String`.
    ///
    /// # Returns
    ///
    /// * [String] - A new `String` whose content is the original content of the `slice`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let source = String::from_ascii_str("Fuel");
    ///     let string = String::from_moved_raw_slice(source.as_raw_slice());
    ///
    ///     // ** `source` must not be used after this point. **
    ///
    ///     assert_eq(string.len(), 4);
    /// }
    /// ```
    pub fn from_moved_raw_slice(slice: raw_slice) -> Self {
        Self {
            bytes: Bytes::from_moved_raw_slice(slice),
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
    ///     let mut string = String::from_ascii_str("Fuel");
    ///     assert(!string.is_empty());
    ///     string.clear();
    ///     assert(string.is_empty());
    ///
    ///     assert(String::new().is_empty());
    /// }
    /// ```
    pub fn is_empty(self) -> bool {
        self.bytes.is_empty()
    }

    /// Constructs a new empty instance of the `String` type.
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
    ///     assert(string.is_empty());
    /// }
    /// ```
    pub fn new() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Constructs a new instance of the `String` type with the specified `capacity`.
    ///
    /// # Arguments
    ///
    /// * `capacity`: [u64] - The specified amount of bytes on the heap to be allocated for the `String`.
    ///
    /// # Returns
    ///
    /// * [String] - A new empty instance of the `String` type with the specified `capacity`.
    ///
    /// # Examples
    ///
    /// ```sway
    /// use std::string::String;
    ///
    /// fn foo() {
    ///     let string = String::with_capacity(1);
    ///     assert_eq(string.capacity(), 1);
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

    /// Gets the length of the `String` in bytes, not chars or graphemes.
    /// In other words, it might not be what a human considers the length of the string.
    ///
    /// # Returns
    ///
    /// * [u64] - The length of the `String` in bytes, not chars or graphemes.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel");
    ///     assert_eq(string.len(), 4);
    /// }
    /// ```
    pub fn len(self) -> u64 {
        self.bytes.len()
    }

    /// Converts the `String` into a string slice.
    ///
    /// # Returns
    ///
    /// [str] - The `String` as a string slice.
    ///
    /// # Examples
    ///
    /// ```sway
    /// fn foo() {
    ///     let string = String::from_ascii_str("Fuel");
    ///     assert(string.as_str() == "Fuel");
    /// }
    /// ```
    pub fn as_str(self) -> str {
        let ptr = self.bytes.ptr();
        let str_size = self.bytes.len();

        __transmute::<(raw_ptr, u64), str>((ptr, str_size))
    }
}

impl From<Bytes> for String {
    fn from(b: Bytes) -> Self {
        Self {
            bytes: b.clone(),
        }
    }
}

impl From<String> for Bytes {
    fn from(s: String) -> Bytes {
        s.as_bytes()
    }
}

impl From<str> for String {
    fn from(s: str) -> String {
        String::from_ascii_str(s)
    }
}

impl From<String> for str {
    fn from(s: String) -> str {
        s.as_str()
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
    /// # Additional Information
    ///
    /// The content of the `slice` gets copied into the newly created `String`
    /// which allocates its own buffer.
    ///
    /// To take the ownership of the `slice` and move it into the newly created
    /// `String` without copying the content, use `String::from_moved_raw_slice`.
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
        s.bytes.as_raw_slice()
    }
}

impl PartialEq for String {
    fn eq(self, other: Self) -> bool {
        self.bytes == other.as_bytes()
    }
}
impl Eq for String {}

impl Hash for String {
    fn hash(self, ref mut state: Hasher) {
        self.bytes.hash(state);
    }
}

impl AbiEncode for String {
    fn is_encode_trivial() -> bool {
        false
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        self.bytes.abi_encode(buffer)
    }
}

impl AbiDecode for String {
    fn is_decode_trivial() -> bool {
        false
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        String {
            bytes: Bytes::abi_decode(buffer),
        }
    }
}

impl Clone for String {
    fn clone(self) -> Self {
        Self {
            bytes: self.bytes.clone(),
        }
    }
}

impl Debug for String {
    fn fmt(self, ref mut f: Formatter) {
        let s = __transmute::<(raw_ptr, u64), str>((self.bytes.ptr(), self.bytes.len()));
        f.print_string_quotes();
        f.print_str(s);
        f.print_string_quotes();
    }
}
