//! Utility functions for cryptographic hashing.
library;

use ::alloc::alloc_bytes;
use ::bytes::*;
use ::codec::*;
use ::debug::*;

pub struct Hasher {
    bytes: Bytes,
}

impl Hasher {
    pub fn new() -> Self {
        Self {
            bytes: Bytes::new(),
        }
    }

    /// Writes some data into this `Hasher`.
    pub fn write(ref mut self, bytes: Bytes) {
        self.bytes.append(bytes);
    }

    pub fn sha256(self) -> b256 {
        let mut result_buffer = b256::min();
        asm(
            hash: result_buffer,
            ptr: self.bytes.ptr(),
            bytes: self.bytes.len(),
        ) {
            s256 hash ptr bytes;
            hash: b256
        }
    }

    pub fn keccak256(self) -> b256 {
        let mut result_buffer = b256::min();
        asm(
            hash: result_buffer,
            ptr: self.bytes.ptr(),
            bytes: self.bytes.len(),
        ) {
            k256 hash ptr bytes;
            hash: b256
        }
    }
}

impl Hasher {
    /// Writes a single `str` into this hasher.
    pub fn write_str(ref mut self, s: str) {
        let str_size = s.len();
        let str_ptr = s.as_ptr();

        self.write(Bytes::from(raw_slice::from_parts::<u8>(str_ptr, str_size)));
    }

    #[inline(never)]
    pub fn write_str_array<S>(ref mut self, s: S) {
        __assert_is_str_array::<S>();
        let str_size = __size_of_str_array::<S>();
        let str_ptr = __addr_of(s);

        self.write(Bytes::from(raw_slice::from_parts::<u8>(str_ptr, str_size)));
    }
}

pub trait Hash {
    fn hash(self, ref mut state: Hasher);
}

impl Hash for u8 {
    fn hash(self, ref mut state: Hasher) {
        let mut bytes = Bytes::with_capacity(1);
        bytes.push(self);
        state.write(bytes);
    }
}

impl Hash for u16 {
    fn hash(self, ref mut state: Hasher) {
        let ptr = alloc_bytes(8); // one word capacity
        asm(ptr: ptr, val: self, r1) {
            slli r1 val i48;
            sw ptr r1 i0;
        };

        state.write(Bytes::from(raw_slice::from_parts::<u8>(ptr, 2)));
    }
}

impl Hash for u32 {
    fn hash(self, ref mut state: Hasher) {
        let ptr = alloc_bytes(8); // one word capacity
        asm(ptr: ptr, val: self, r1) {
            slli r1 val i32;
            sw ptr r1 i0;
        };

        state.write(Bytes::from(raw_slice::from_parts::<u8>(ptr, 4)));
    }
}

impl Hash for u64 {
    fn hash(self, ref mut state: Hasher) {
        let ptr = alloc_bytes(8); // one word capacity
        asm(ptr: ptr, val: self) {
            sw ptr val i0;
        };

        state.write(Bytes::from(raw_slice::from_parts::<u8>(ptr, 8)));
    }
}

impl Hash for b256 {
    fn hash(self, ref mut state: Hasher) {
        let ptr = alloc_bytes(32); // four word capacity
        let (word_1, word_2, word_3, word_4) = asm(r1: self) {
            r1: (u64, u64, u64, u64)
        };

        asm(
            ptr: ptr,
            val_1: word_1,
            val_2: word_2,
            val_3: word_3,
            val_4: word_4,
        ) {
            sw ptr val_1 i0;
            sw ptr val_2 i1;
            sw ptr val_3 i2;
            sw ptr val_4 i3;
        };

        state.write(Bytes::from(raw_slice::from_parts::<u8>(ptr, 32)));
    }
}

impl Hash for u256 {
    fn hash(self, ref mut state: Hasher) {
        let ptr = alloc_bytes(32); // four word capacity
        let (word_1, word_2, word_3, word_4) = asm(r1: self) {
            r1: (u64, u64, u64, u64)
        };

        asm(
            ptr: ptr,
            val_1: word_1,
            val_2: word_2,
            val_3: word_3,
            val_4: word_4,
        ) {
            sw ptr val_1 i0;
            sw ptr val_2 i1;
            sw ptr val_3 i2;
            sw ptr val_4 i3;
        };

        state.write(Bytes::from(raw_slice::from_parts::<u8>(ptr, 32)));
    }
}

impl Hash for bool {
    fn hash(self, ref mut state: Hasher) {
        let mut bytes = Bytes::with_capacity(1);
        if self {
            bytes.push(1_u8);
        } else {
            bytes.push(0_u8);
        }
        state.write(bytes);
    }
}

impl Hash for Bytes {
    fn hash(self, ref mut state: Hasher) {
        state.write(self);
    }
}

impl Hash for str {
    fn hash(self, ref mut state: Hasher) {
        state.write_str(self);
    }
}

impl<A, B> Hash for (A, B)
where
    A: Hash,
    B: Hash,
{
    #[inline(never)]
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<A, B, C> Hash for (A, B, C)
where
    A: Hash,
    B: Hash,
    C: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
    }
}

impl<A, B, C, D> Hash for (A, B, C, D)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
    }
}

impl<A, B, C, D, E> Hash for (A, B, C, D, E)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
        self.4.hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 1]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 2]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 3]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 4]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 5]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 6]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 7]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
        self[6].hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 8]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
        self[6].hash(state);
        self[7].hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 9]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
        self[6].hash(state);
        self[7].hash(state);
        self[8].hash(state);
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 10]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
        self[6].hash(state);
        self[7].hash(state);
        self[8].hash(state);
        self[9].hash(state);
    }
}

#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> Hash for [T; N]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        let mut i = 0;
        while __lt(i, N) {
            let item: T = *__elem_at(&self, i);
            item.hash(state);
            i = __add(i, 1);
        }
    }
}

/// Returns the `SHA-2-256` hash of `param`.
///
/// # Arguments
///
/// * `s`: [T] - The value to be hashed.
///
/// # Returns
///
/// * [b256] - The sha-256 hash of the value.
///
/// # Examples
///
/// ```sway
/// use std::hash::*;
///
/// fn foo() {
///     let result = sha256("Fuel");
///     assert(result == 0xa80f942f4112036dfc2da86daf6d2ef6ede3164dd56d1000eb82fa87c992450f);
/// }
/// ```
#[inline(never)]
pub fn sha256<T>(s: T) -> b256
where
    T: Hash,
{
    let mut hasher = Hasher::new();
    s.hash(hasher);
    hasher.sha256()
}

/// Returns the `SHA-2-256` hash of `param`.
/// This function is specific for string arrays
///
/// # Examples
///
/// ```sway
/// use std::hash::*;
///
/// fn foo() {
///     let result = sha256_str_array(__to_str_array("Fuel"));
///     assert(result == 0xa80f942f4112036dfc2da86daf6d2ef6ede3164dd56d1000eb82fa87c992450f);
/// }
/// ```
#[inline(never)]
pub fn sha256_str_array<S>(param: S) -> b256 {
    __assert_is_str_array::<S>();
    let str_size = __size_of_str_array::<S>();
    let str_ptr = __addr_of(param);

    let ptr = alloc_bytes(str_size);
    str_ptr.copy_bytes_to(ptr, str_size);

    let mut hasher = Hasher::new();
    hasher.write(Bytes::from(raw_slice::from_parts::<u8>(ptr, str_size)));
    hasher.sha256()
}

/// Returns the `KECCAK-256` hash of `param`.
///
/// # Arguments
///
/// * `s`: [T] - The value to be hashed.
///
/// # Returns
///
/// * [b256] - The keccak-256 hash of the value.
///
/// # Examples
///
/// ```sway
/// use std::hash::keccak256;
///
/// fn foo() {
///     let result = keccak256("Fuel");
///     assert(result == 0x4375c8bcdc904e5f51752581202ae9ae2bb6eddf8de05d5567d9a6b0ae4789ad);
/// }
/// ```
#[inline(never)]
pub fn keccak256<T>(s: T) -> b256
where
    T: Hash,
{
    let mut hasher = Hasher::new();
    s.hash(hasher);
    hasher.keccak256()
}

#[cfg(experimental_const_generics = true)]
#[test]
fn ok_array_hash() {
    use ::ops::*;

    let a = sha256([1, 2, 3]);
    let b = sha256((1, 2, 3));

    if a != b {
        __revert(0);
    }
}
