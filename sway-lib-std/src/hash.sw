//! Utility functions for cryptographic hashing.
library;

use ::option::Option;
use ::result::Result;
use ::vec::Vec;
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

    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            bytes: Bytes::with_capacity(capacity),
        }
    }

    /// Appends content of `bytes` to this `Hasher`.
    ///
    /// Note that the length of `bytes` is not appended to the
    /// `Hasher`, just the content.
    pub fn write(ref mut self, bytes: Bytes) {
        self.bytes.append(bytes);
    }

    /// Appends bytes from the `slice` to this `Hasher`.
    ///
    /// Note that the length of `slice` is not appended to the
    /// `Hasher`, just the bytes within the `slice`.
    pub fn write_raw_slice(ref mut self, slice: raw_slice) {
        self.bytes.append_raw_slice(slice);
    }

    /// Appends `u8` `value` to this `Hasher`.
    pub fn write_u8(ref mut self, value: u8) {
        self.bytes.push(value);
    }

    /// Appends a single `str` to this `Hasher`.
    ///
    /// Note that the length of `s` is not appended to the
    /// `Hasher`, just the bytes forming the string `s`.
    pub fn write_str(ref mut self, s: str) {
        self.bytes
            .append_raw_slice(raw_slice::from_parts::<u8>(s.as_ptr(), s.len()));
    }

    /// Appends a single string array to this `Hasher`.
    ///
    /// Note that the length of `s` is not appended to the
    /// `Hasher`, just the bytes forming the string array `s`.
    #[inline(never)]
    pub fn write_str_array<S>(ref mut self, s: S) {
        __assert_is_str_array::<S>();
        let str_size = __size_of_str_array::<S>();
        let str_ptr = __addr_of(s);

        self.bytes
            .append_raw_slice(raw_slice::from_parts::<u8>(str_ptr, str_size));
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

pub trait Hash {
    fn hash(self, ref mut state: Hasher);
}

impl Hash for u8 {
    fn hash(self, ref mut state: Hasher) {
        state.write_u8(self);
    }
}

impl Hash for u16 {
    fn hash(self, ref mut state: Hasher) {
        // TODO: Remove this workaround once `__addr_of(self)` is supported
        //       for `u16`.
        let temp = self;
        let ptr = asm(ptr: &temp) {
            ptr: raw_ptr
        };
        let ptr = ptr.add::<u8>(6);

        state.write_raw_slice(raw_slice::from_parts::<u8>(ptr, 2));
    }
}

impl Hash for u32 {
    fn hash(self, ref mut state: Hasher) {
        // TODO: Remove this workaround once `__addr_of(self)` is supported
        //       for `u32`.
        let temp = self;
        let ptr = asm(ptr: &temp) {
            ptr: raw_ptr
        };
        let ptr = ptr.add::<u8>(4);

        state.write_raw_slice(raw_slice::from_parts::<u8>(ptr, 4));
    }
}

impl Hash for u64 {
    fn hash(self, ref mut state: Hasher) {
        // TODO: Remove this workaround once `__addr_of(self)` is supported
        //       for `u64`.
        let temp = self;
        let ptr = asm(ptr: &temp) {
            ptr: raw_ptr
        };

        state.write_raw_slice(raw_slice::from_parts::<u8>(ptr, 8));
    }
}

impl Hash for b256 {
    fn hash(self, ref mut state: Hasher) {
        state.write_raw_slice(raw_slice::from_parts::<u8>(__addr_of(self), 32));
    }
}

impl Hash for u256 {
    fn hash(self, ref mut state: Hasher) {
        state.write_raw_slice(raw_slice::from_parts::<u8>(__addr_of(self), 32));
    }
}

impl Hash for bool {
    fn hash(self, ref mut state: Hasher) {
        state.write_u8(if self { 1_u8 } else { 0_u8 });
    }
}

#[cfg(experimental_new_hashing = false)]
impl Hash for Bytes {
    fn hash(self, ref mut state: Hasher) {
        state.write(self);
    }
}

#[cfg(experimental_new_hashing = true)]
impl Hash for Bytes {
    fn hash(self, ref mut state: Hasher) {
        self.len().hash(state);
        state.write(self);
    }
}

#[cfg(experimental_new_hashing = false)]
impl Hash for str {
    fn hash(self, ref mut state: Hasher) {
        state.write_str(self);
    }
}

#[cfg(experimental_new_hashing = true)]
impl Hash for str {
    fn hash(self, ref mut state: Hasher) {
        self.len().hash(state);
        state.write_str(self);
    }
}

impl Hash for () {
    fn hash(self, ref mut _state: Hasher) {}
}

impl<A> Hash for (A, )
where
    A: Hash,
{
    #[inline(never)]
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
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

impl<A, B, C, D, E, F> Hash for (A, B, C, D, E, F)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
        self.4.hash(state);
        self.5.hash(state);
    }
}

impl<A, B, C, D, E, F, G> Hash for (A, B, C, D, E, F, G)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
    G: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
        self.4.hash(state);
        self.5.hash(state);
        self.6.hash(state);
    }
}

impl<A, B, C, D, E, F, G, H> Hash for (A, B, C, D, E, F, G, H)
where
    A: Hash,
    B: Hash,
    C: Hash,
    D: Hash,
    E: Hash,
    F: Hash,
    G: Hash,
    H: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self.0.hash(state);
        self.1.hash(state);
        self.2.hash(state);
        self.3.hash(state);
        self.4.hash(state);
        self.5.hash(state);
        self.6.hash(state);
        self.7.hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 0]
where
    T: Hash,
{
    fn hash(self, ref mut _state: Hasher) {}
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 0]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        0_u64.hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 1]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        self[0].hash(state);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 1]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        1_u64.hash(state);
        self[0].hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 2]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        2_u64.hash(state);
        self[0].hash(state);
        self[1].hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 3]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        3_u64.hash(state);
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 4]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        4_u64.hash(state);
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 5]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        5_u64.hash(state);
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 6]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        6_u64.hash(state);
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 7]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        7_u64.hash(state);
        self[0].hash(state);
        self[1].hash(state);
        self[2].hash(state);
        self[3].hash(state);
        self[4].hash(state);
        self[5].hash(state);
        self[6].hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 8]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        8_u64.hash(state);
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

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 9]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        9_u64.hash(state);
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

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl<T> Hash for [T; 10]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        10_u64.hash(state);
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

#[cfg(experimental_new_hashing = false)]
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

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> Hash for [T; N]
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        N.hash(state);
        let mut i = 0;
        while __lt(i, N) {
            let item: T = *__elem_at(&self, i);
            item.hash(state);
            i = __add(i, 1);
        }
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[0] {
    fn hash(self, ref mut _state: Hasher) {}
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[0] {
    fn hash(self, ref mut state: Hasher) {
        0_u64.hash(state);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[1] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[1] {
    fn hash(self, ref mut state: Hasher) {
        1_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[2] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[2] {
    fn hash(self, ref mut state: Hasher) {
        2_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[3] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[3] {
    fn hash(self, ref mut state: Hasher) {
        3_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[4] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[4] {
    fn hash(self, ref mut state: Hasher) {
        4_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[5] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[5] {
    fn hash(self, ref mut state: Hasher) {
        5_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[6] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[6] {
    fn hash(self, ref mut state: Hasher) {
        6_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[7] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[7] {
    fn hash(self, ref mut state: Hasher) {
        7_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[8] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[8] {
    fn hash(self, ref mut state: Hasher) {
        8_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[9] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[9] {
    fn hash(self, ref mut state: Hasher) {
        9_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[10] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = false)]
impl Hash for str[10] {
    fn hash(self, ref mut state: Hasher) {
        10_u64.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = true)]
impl<const N: u64> Hash for str[N] {
    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = true)]
impl<const N: u64> Hash for str[N] {
    fn hash(self, ref mut state: Hasher) {
        N.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
impl Hash for raw_slice {
    fn hash(self, ref mut state: Hasher) {
        state.write_raw_slice(self);
    }
}

#[cfg(experimental_new_hashing = true)]
impl Hash for raw_slice {
    fn hash(self, ref mut state: Hasher) {
        self.number_of_bytes().hash(state);
        state.write_raw_slice(self);
    }
}

#[cfg(experimental_new_hashing = false)]
impl<T> Hash for Vec<T>
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        let len = self.len();
        // `__elem_at` accepts only a reference to a slice or an array.
        // To satisfy this requirement, we cast the pointer to the underlying
        // vector data to an array reference.
        let ptr = asm(ptr: self.ptr()) {
            ptr: &[T; 0]
        };

        let mut i = 0;
        while __lt(i, len) {
            let item: T = *__elem_at(ptr, i);
            item.hash(state);
            i = __add(i, 1);
        }
    }
}

#[cfg(experimental_new_hashing = true)]
impl<T> Hash for Vec<T>
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        let len = self.len();

        len.hash(state);

        // `__elem_at` accepts only a reference to a slice or an array.
        // To satisfy this requirement, we cast the pointer to the underlying
        // vector data to an array reference.
        let ptr = asm(ptr: self.ptr()) {
            ptr: &[T; 0]
        };

        let mut i = 0;
        while __lt(i, len) {
            let item: T = *__elem_at(ptr, i);
            item.hash(state);
            i = __add(i, 1);
        }
    }
}

impl<T> Hash for Option<T>
where
    T: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        match self {
            Self::None => {
                0_u8.hash(state);
            },
            Self::Some(v) => {
                1_u8.hash(state);
                v.hash(state);
            },
        }
    }
}

impl<T, E> Hash for Result<T, E>
where
    T: Hash,
    E: Hash,
{
    fn hash(self, ref mut state: Hasher) {
        match self {
            Self::Ok(v) => {
                0_u8.hash(state);
                v.hash(state);
            },
            Self::Err(err) => {
                1_u8.hash(state);
                err.hash(state);
            },
        }
    }
}

/// Returns the `Hasher`'s initial capacity optimal for hashing
/// an instance of type `T`.
fn get_initial_capacity<T>() -> u64 {
    if __is_str_array::<T>() {
        __size_of_str_array::<T>()
    } else {
        // This will work accurately for all non-heap types.
        // For heap types, it still gives a slightly better
        // start then having the empty buffer.
        __size_of::<T>()
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
    // TODO: Replace `capacity` with a compile-time constant once
    //       `const fn` is implemented and const evaluation is
    //       deferred for generic functions:
    //
    //       const CAPACITY: u64 = get_initial_capacity::<T>();
    let capacity = get_initial_capacity::<T>();
    let mut hasher = Hasher::with_capacity(capacity);
    s.hash(hasher);
    hasher.sha256()
}

/// Returns the `SHA-2-256` hash of `param`.
/// This function is specific for string arrays.
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
    // TODO: Replace `capacity` with a compile-time constant once
    //       `const fn` is implemented and const evaluation is
    //       deferred for generic functions:
    //
    //       const CAPACITY: u64 = get_initial_capacity::<S>();
    let capacity = get_initial_capacity::<S>();
    let mut hasher = Hasher::with_capacity(capacity);
    hasher.write_str_array(param);
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
    // TODO: Replace `capacity` with a compile-time constant once
    //       `const fn` is implemented and const evaluation is
    //       deferred for generic functions:
    //
    //       const CAPACITY: u64 = get_initial_capacity::<T>();
    let capacity = get_initial_capacity::<T>();
    let mut hasher = Hasher::with_capacity(capacity);
    s.hash(hasher);
    hasher.keccak256()
}

#[cfg(experimental_new_hashing = false)]
#[cfg(experimental_const_generics = true)]
#[test]
fn ok_array_hash() {
    use ::ops::*;
    use ::assert::*;

    // Arrays and tuples
    let a = sha256([1, 2, 3]);
    let b = sha256((1, 2, 3));
    assert(a == b);

    // string slices
    let a = sha256(("abc", "def"));
    let b = sha256(("ab", "cd", "ef"));
    assert(a == b);

    // string arrays
    let a = sha256((__to_str_array("abc"), __to_str_array("def")));
    let b = sha256((__to_str_array("ab"), __to_str_array("cd"), __to_str_array("ef")));
    assert(a == b);
}

#[cfg(experimental_new_hashing = true)]
#[cfg(experimental_const_generics = true)]
#[test]
fn ok_array_hash() {
    use ::ops::*;
    use ::assert::*;

    // Arrays and tuples
    let a = sha256([1, 2, 3]);
    let b = sha256((1, 2, 3));
    assert(a != b);
    let b = sha256((3_u64, 1, 2, 3));
    assert(a == b);

    // string slices
    let a = sha256(("abc", "def"));
    let b = sha256(("ab", "cd", "ef"));
    assert(a != b);
    let b = sha256((3u64, 97u8, 98u8, 99u8, 3u64, 100u8, 101u8, 102u8));
    assert(a == b);

    // string arrays
    let a = sha256((__to_str_array("abc"), __to_str_array("def")));
    let b = sha256((__to_str_array("ab"), __to_str_array("cd"), __to_str_array("ef")));
    assert(a != b);
    let b = sha256((3u64, 97u8, 98u8, 99u8, 3u64, 100u8, 101u8, 102u8));
    assert(a == b);
}
