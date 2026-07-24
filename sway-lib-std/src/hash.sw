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

/// Trait for hashing values into a [Hasher].
///
/// Types implementing `Hash` can be hashed using the [Hasher], or via the
/// [sha256] and [keccak256] convenience functions, and can be used in places
/// that require a deterministic hash, e.g., as keys in a `StorageMap`.
///
/// # Implementing `Hash`
///
/// A `Hash` implementation defines how the value is written into the [Hasher]
/// via the [Hash::hash] method, and declares, via [Hash::is_hash_trivial],
/// whether the value's in-memory representation is byte-for-byte identical to
/// the bytes it writes into the [Hasher].
///
/// ```sway
/// use std::hash::{Hash, Hasher};
///
/// struct MyU64Wrapper {
///     value: u64,
/// }
///
/// impl Hash for MyU64Wrapper {
///     fn is_hash_trivial() -> bool {
///         // A struct with a single `u64` field has the same in-memory
///         // representation as its hash bytes representation.
///         true
///     }
///
///     fn hash(self, ref mut state: Hasher) {
///         self.value.hash(state);
///     }
/// }
/// ```
pub trait Hash {
    /// Returns `true` if the *in-memory representation* of the type is
    /// byte-for-byte identical to the *hash bytes representation* that
    /// [Hash::hash] writes into the [Hasher]; otherwise, returns `false`.
    ///
    /// # Additional Information
    ///
    /// When a type is trivially hashable, its hash can be computed by using
    /// the raw memory of the value (`__size_of::<Self>()` bytes starting at the
    /// value's address), without first building an intermediate byte buffer
    /// in a [Hasher]. This is what the [sha256] and [keccak256] functions do
    /// for trivially hashable types, which is significantly more gas effective.
    ///
    /// **Returning `true` is a strong guarantee: an incorrect `true` will produce
    /// wrong hashes when a value is hashed via [sha256] or [keccak256].** When in
    /// doubt, return `false`. Returning `false` is always safe; it only forgoes
    /// the optimization.
    ///
    /// # Warning
    ///
    /// Several subtleties make types that might look trivially hashable actually
    /// **not** trivially hashable. In particular:
    ///
    /// - `u16` and `u32` are stored in memory in an eight-byte slot (as `u64`),
    ///   but their hash bytes representation is only two and four bytes,
    ///   respectively. They are therefore **never** trivially hashable, and
    ///   neither is any aggregate (struct, tuple, array, ...) containing them.
    /// - `bool`, `u8`, `u16`, and `u32` fields inside a struct or tuple are
    ///   padded to eight bytes in memory. An aggregate containing such a field
    ///   is therefore **not** trivially hashable, even though the field types
    ///   themselves might be when hashed on their own.
    /// - Enum tags are stored in memory as `u64`, but the `Hash` implementations
    ///   in this library hash them as `u8`. Enums are therefore **not** trivially
    ///   hashable.
    /// - Collections (`Bytes`, `Vec`, `raw_slice`, `str`, `str[N]`, arrays, and
    ///   aggregates containing them) can be trivially hashable or not depending
    ///   on the `new_hashing` experimental feature. When `new_hashing` is
    ///   enabled, collections prefix their content with their length, which means
    ///   their hash bytes representation no longer matches their in-memory
    ///   representation, making them **not** trivially hashable.
    ///   For more details see: https://github.com/FuelLabs/sway/issues/7256.
    // TODO-MEMLAY: In this doc-comment we make an assumption about the memory
    //              layouts. Those can be changed in the future.
    // TODO: (TRIVIALLY-HASHABLE-ENUMS) Once we change enum implementations, adapt
    //       the doc-comment.
    fn is_hash_trivial() -> bool;

    /// Writes the hash bytes representation of `self` into the [Hasher] `state`.
    fn hash(self, ref mut state: Hasher);
}

/// Returns `true` if the type `T` is trivially hashable.
///
/// See [Hash::is_hash_trivial] for the exact meaning and the caveats.
pub fn is_hash_trivial<T>() -> bool
where
    T: Hash,
{
    T::is_hash_trivial()
}

impl Hash for u8 {
    fn is_hash_trivial() -> bool {
        true
    }

    fn hash(self, ref mut state: Hasher) {
        state.write_u8(self);
    }
}

impl Hash for u16 {
    fn is_hash_trivial() -> bool {
        // `u16` is stored in memory in an eight-byte slot, but hashed as two bytes.
        false
    }

    fn hash(self, ref mut state: Hasher) {
        let ptr = __addr_of(self).add::<u8>(6);
        state.write_raw_slice(raw_slice::from_parts::<u8>(ptr, 2));
    }
}

impl Hash for u32 {
    fn is_hash_trivial() -> bool {
        // `u32` is stored in memory in an eight-byte slot, but hashed as four bytes.
        false
    }

    fn hash(self, ref mut state: Hasher) {
        let ptr = __addr_of(self).add::<u8>(4);
        state.write_raw_slice(raw_slice::from_parts::<u8>(ptr, 4));
    }
}

impl Hash for u64 {
    fn is_hash_trivial() -> bool {
        true
    }

    fn hash(self, ref mut state: Hasher) {
        state.write_raw_slice(raw_slice::from_parts::<u8>(__addr_of(self), 8));
    }
}

impl Hash for b256 {
    fn is_hash_trivial() -> bool {
        true
    }

    fn hash(self, ref mut state: Hasher) {
        state.write_raw_slice(raw_slice::from_parts::<u8>(__addr_of(self), 32));
    }
}

impl Hash for u256 {
    fn is_hash_trivial() -> bool {
        true
    }

    fn hash(self, ref mut state: Hasher) {
        state.write_raw_slice(raw_slice::from_parts::<u8>(__addr_of(self), 32));
    }
}

impl Hash for bool {
    fn is_hash_trivial() -> bool {
        true
    }

    fn hash(self, ref mut state: Hasher) {
        state.write_u8(if self { 1_u8 } else { 0_u8 });
    }
}

#[cfg(experimental_new_hashing = false)]
impl Hash for Bytes {
    fn is_hash_trivial() -> bool {
        false
    }

    fn hash(self, ref mut state: Hasher) {
        state.write(self);
    }
}

#[cfg(experimental_new_hashing = true)]
impl Hash for Bytes {
    fn is_hash_trivial() -> bool {
        false
    }

    fn hash(self, ref mut state: Hasher) {
        self.len().hash(state);
        state.write(self);
    }
}

#[cfg(experimental_new_hashing = false)]
impl Hash for str {
    fn is_hash_trivial() -> bool {
        // `str` is a fat pointer containing the location and length.
        // Its in-memory representation never matches its hash bytes.
        false
    }

    fn hash(self, ref mut state: Hasher) {
        state.write_str(self);
    }
}

#[cfg(experimental_new_hashing = true)]
impl Hash for str {
    fn is_hash_trivial() -> bool {
        // `str` is a fat pointer containing the location and length.
        // Its in-memory representation never matches its hash bytes.
        false
    }

    fn hash(self, ref mut state: Hasher) {
        self.len().hash(state);
        state.write_str(self);
    }
}

impl Hash for () {
    fn is_hash_trivial() -> bool {
        true
    }

    fn hash(self, ref mut _state: Hasher) {}
}

// For tuples, aside from requiring all elements to be trivially hashable,
// we have to make sure that there are no paddings between the elements.
// For that, we use here the equality of the `__mem_repr_id_runtime` and the
// `__mem_repr_id_hashing`.

impl<A> Hash for (A, )
where
    A: Hash,
{
    fn is_hash_trivial() -> bool {
        let r = __mem_repr_id_runtime::<Self>() == __mem_repr_id_hashing::<Self>();
        let r = r && is_hash_trivial::<A>();
        r
    }

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
    fn is_hash_trivial() -> bool {
        let r = __mem_repr_id_runtime::<Self>() == __mem_repr_id_hashing::<Self>();
        let r = r && is_hash_trivial::<A>();
        let r = r && is_hash_trivial::<B>();
        r
    }

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
    fn is_hash_trivial() -> bool {
        let r = __mem_repr_id_runtime::<Self>() == __mem_repr_id_hashing::<Self>();
        let r = r && is_hash_trivial::<A>();
        let r = r && is_hash_trivial::<B>();
        let r = r && is_hash_trivial::<C>();
        r
    }

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
    fn is_hash_trivial() -> bool {
        let r = __mem_repr_id_runtime::<Self>() == __mem_repr_id_hashing::<Self>();
        let r = r && is_hash_trivial::<A>();
        let r = r && is_hash_trivial::<B>();
        let r = r && is_hash_trivial::<C>();
        let r = r && is_hash_trivial::<D>();
        r
    }

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
    fn is_hash_trivial() -> bool {
        let r = __mem_repr_id_runtime::<Self>() == __mem_repr_id_hashing::<Self>();
        let r = r && is_hash_trivial::<A>();
        let r = r && is_hash_trivial::<B>();
        let r = r && is_hash_trivial::<C>();
        let r = r && is_hash_trivial::<D>();
        let r = r && is_hash_trivial::<E>();
        r
    }

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
    fn is_hash_trivial() -> bool {
        let r = __mem_repr_id_runtime::<Self>() == __mem_repr_id_hashing::<Self>();
        let r = r && is_hash_trivial::<A>();
        let r = r && is_hash_trivial::<B>();
        let r = r && is_hash_trivial::<C>();
        let r = r && is_hash_trivial::<D>();
        let r = r && is_hash_trivial::<E>();
        let r = r && is_hash_trivial::<F>();
        r
    }

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
    fn is_hash_trivial() -> bool {
        let r = __mem_repr_id_runtime::<Self>() == __mem_repr_id_hashing::<Self>();
        let r = r && is_hash_trivial::<A>();
        let r = r && is_hash_trivial::<B>();
        let r = r && is_hash_trivial::<C>();
        let r = r && is_hash_trivial::<D>();
        let r = r && is_hash_trivial::<E>();
        let r = r && is_hash_trivial::<F>();
        let r = r && is_hash_trivial::<G>();
        r
    }

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
    fn is_hash_trivial() -> bool {
        let r = __mem_repr_id_runtime::<Self>() == __mem_repr_id_hashing::<Self>();
        let r = r && is_hash_trivial::<A>();
        let r = r && is_hash_trivial::<B>();
        let r = r && is_hash_trivial::<C>();
        let r = r && is_hash_trivial::<D>();
        let r = r && is_hash_trivial::<E>();
        let r = r && is_hash_trivial::<F>();
        let r = r && is_hash_trivial::<G>();
        let r = r && is_hash_trivial::<H>();
        r
    }

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
impl<T, const N: u64> Hash for [T; N]
where
    T: Hash,
{
    fn is_hash_trivial() -> bool {
        // When `new_hashing` is false no length prefix is written
        // and an array is trivially hashable if its elements are
        // trivially hashable.
        is_hash_trivial::<T>()
    }

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
impl<T, const N: u64> Hash for [T; N]
where
    T: Hash,
{
    fn is_hash_trivial() -> bool {
        // With `new_hashing` enabled, the array is prefixed with its length,
        // so its hash bytes no longer match its in-memory representation.
        false
    }

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
impl<const N: u64> Hash for str[N] {
    fn is_hash_trivial() -> bool {
        // A string array is trivially hashable only when it has no trailing
        // padding, so that its in-memory bytes exactly match the hashed
        // characters. If `new_hashing` is false no length prefix is
        // written, so the hashed bytes are exactly the string array bytes.
        //
        // This is precisely the condition under which a string array is
        // trivially ABI-encodeable because in this mode a string array is
        // hashed exactly as it is encoded, so we reuse that condition here.
        //
        // Note that, in a way, we are coupling here trivial hashing and
        // trivial encoding. On the other side, the semantics of trivial
        // encoding for `str[N]` will not change and reusing it here gives
        // simpler approach than adding additional `cfg` conditions.
        is_encode_trivial::<str[N]>()
    }

    fn hash(self, ref mut state: Hasher) {
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = true)]
impl<const N: u64> Hash for str[N] {
    fn is_hash_trivial() -> bool {
        // With `new_hashing` enabled, the string array is prefixed with its
        // length, so its hash bytes no longer match its in-memory representation.
        false
    }

    fn hash(self, ref mut state: Hasher) {
        N.hash(state);
        state.write_str_array(self);
    }
}

#[cfg(experimental_new_hashing = false)]
impl Hash for raw_slice {
    fn is_hash_trivial() -> bool {
        // `raw_slice` is a fat pointer containing the location and length.
        // Its in-memory representation never matches its hash bytes.
        false
    }

    fn hash(self, ref mut state: Hasher) {
        state.write_raw_slice(self);
    }
}

#[cfg(experimental_new_hashing = true)]
impl Hash for raw_slice {
    fn is_hash_trivial() -> bool {
        // `raw_slice` is a fat pointer containing the location and length.
        // Its in-memory representation never matches its hash bytes.
        false
    }

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
    fn is_hash_trivial() -> bool {
        false
    }

    fn hash(self, ref mut state: Hasher) {
        let len = self.len();
        // `__elem_at` accepts only a reference to a slice or an array.
        // To satisfy this requirement, we cast the pointer to the underlying
        // vector data to an array reference.
        let ptr = __transmute::<raw_ptr, &[T; 0]>(self.ptr());

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
    fn is_hash_trivial() -> bool {
        false
    }

    fn hash(self, ref mut state: Hasher) {
        let len = self.len();

        len.hash(state);

        // `__elem_at` accepts only a reference to a slice or an array.
        // To satisfy this requirement, we cast the pointer to the underlying
        // vector data to an array reference.
        let ptr = __transmute::<raw_ptr, &[T; 0]>(self.ptr());

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
    fn is_hash_trivial() -> bool {
        false
    }

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
    fn is_hash_trivial() -> bool {
        // TODO: (HASH-TRIVIAL-ENUMS) The enum tag is stored in memory as a `u64`,
        //       but hashed as a `u8`.
        //       Otherwise we can have triviality iff:
        //        - `T` and `E` have the same size
        //        - `T` and `E` are both trivial
        //        - runtime and packed mem repr is same, for both `T` and `E`
        false
    }

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
#[cfg(experimental_new_hashing = false)]
fn get_initial_capacity<T>() -> u64 {
    if __is_str_array::<T>() {
        __size_of_str_array::<T>()
    } else {
        // This will work accurately for all non-dynamic types.
        // For dynamic types, it might give a slightly better
        // start then having an empty buffer, or a useless
        // initial allocation, depending on the size of the
        // content.
        __size_of::<T>()
    }
}

/// Returns the `Hasher`'s initial capacity optimal for hashing
/// an instance of type `T`.
#[cfg(experimental_new_hashing = true)]
fn get_initial_capacity<T>() -> u64 {
    if __is_str_array::<T>() {
        // Add 8 bytes for the length prefix.
        __size_of_str_array::<T>() + 8
    } else {
        // This will work accurately for all non-dynamic types.
        // For dynamic types, it might give a slightly better
        // start then having an empty buffer, or a useless
        // initial allocation, depending on the size of the
        // content.
        __size_of::<T>()
    }
}

/// Returns the `SHA-2-256` hash of `val`.
///
/// # Arguments
///
/// * `val`: [T] - The value to be hashed.
///
/// # Returns
///
/// * [b256] - The sha-256 hash of the `val`.
///
/// # Examples
///
/// ```sway
/// use std::hash::*;
///
/// fn foo() {
///     let result = sha256("Fuel");
///     assert_eq(result, 0xa80f942f4112036dfc2da86daf6d2ef6ede3164dd56d1000eb82fa87c992450f);
/// }
/// ```
#[inline(never)]
pub fn sha256<T>(val: T) -> b256
where
    T: Hash,
{
    const IS_TRIVIAL: bool = is_hash_trivial::<T>();
    if IS_TRIVIAL {
        // The in-memory representation of `val` is byte-for-byte identical to its
        // hash bytes representation, so we can hash the raw memory directly and
        // avoid building an intermediate buffer in a `Hasher`.
        let mut result_buffer = b256::zero();
        asm(
            hash: result_buffer,
            ptr: __addr_of(val),
            bytes: __size_of::<T>(),
        ) {
            s256 hash ptr bytes;
            hash: b256
        }
    } else {
        // TODO: Replace `capacity` with a compile-time constant once
        //       `const fn` is implemented and const evaluation is
        //       deferred for generic functions:
        //
        //       const CAPACITY: u64 = get_initial_capacity::<T>();
        let capacity = get_initial_capacity::<T>();
        let mut hasher = Hasher::with_capacity(capacity);
        val.hash(hasher);
        hasher.sha256()
    }
}

/// Returns the `SHA-2-256` hash of `val`.
/// This function is specific for string arrays.
///
/// # Examples
///
/// ```sway
/// use std::hash::*;
///
/// fn foo() {
///     let result = sha256_str_array(__to_str_array("Fuel"));
///     assert_eq(result, 0xa80f942f4112036dfc2da86daf6d2ef6ede3164dd56d1000eb82fa87c992450f);
/// }
/// ```
#[cfg(experimental_new_hashing = false)]
#[inline(never)]
pub fn sha256_str_array<S>(val: S) -> b256 {
    // TODO: Replace `capacity` with a compile-time constant once
    //       `const fn` is implemented and const evaluation is
    //       deferred for generic functions:
    //
    //       const CAPACITY: u64 = get_initial_capacity::<S>();
    let capacity = get_initial_capacity::<S>();
    let mut hasher = Hasher::with_capacity(capacity);
    hasher.write_str_array(val);
    hasher.sha256()
}

/// Returns the `SHA-2-256` hash of `val`.
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
#[cfg(experimental_new_hashing = true)]
#[inline(never)]
pub fn sha256_str_array<S>(val: S) -> b256 {
    // TODO: Replace `capacity` with a compile-time constant once
    //       `const fn` is implemented and const evaluation is
    //       deferred for generic functions:
    //
    //       const CAPACITY: u64 = get_initial_capacity::<S>();
    let capacity = get_initial_capacity::<S>();
    let mut hasher = Hasher::with_capacity(capacity);
    __size_of_str_array::<S>().hash(hasher);
    hasher.write_str_array(val);
    hasher.sha256()
}

/// Returns the `KECCAK-256` hash of `val`.
///
/// # Arguments
///
/// * `s`: [T] - The value to be hashed.
///
/// # Returns
///
/// * [b256] - The keccak-256 hash of the `val`.
///
/// # Examples
///
/// ```sway
/// use std::hash::keccak256;
///
/// fn foo() {
///     let result = keccak256("Fuel");
///     assert_eq(result, 0x4375c8bcdc904e5f51752581202ae9ae2bb6eddf8de05d5567d9a6b0ae4789ad);
/// }
/// ```
#[inline(never)]
pub fn keccak256<T>(val: T) -> b256
where
    T: Hash,
{
    const IS_TRIVIAL: bool = is_hash_trivial::<T>();
    if IS_TRIVIAL {
        // The in-memory representation of `val` is byte-for-byte identical to its
        // hash bytes representation, so we can hash the raw memory directly and
        // avoid building an intermediate buffer in a `Hasher`.
        let mut result_buffer = b256::min();
        asm(
            hash: result_buffer,
            ptr: __addr_of(val),
            bytes: __size_of::<T>(),
        ) {
            k256 hash ptr bytes;
            hash: b256
        }
    } else {
        // TODO: Replace `capacity` with a compile-time constant once
        //       `const fn` is implemented and const evaluation is
        //       deferred for generic functions:
        //
        //       const CAPACITY: u64 = get_initial_capacity::<T>();
        let capacity = get_initial_capacity::<T>();
        let mut hasher = Hasher::with_capacity(capacity);
        val.hash(hasher);
        hasher.keccak256()
    }
}
