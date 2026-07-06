library;

use ::ops::*;
use ::raw_ptr::*;
use ::raw_slice::*;
use ::slice::*;
use ::error_signals::*;

pub struct Buffer {
    buffer: (raw_ptr, u64, u64), // ptr, capacity, size
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buffer: __encode_buffer_empty(),
        }
    }

    pub fn with_capacity(cap: u64) -> Self {
        let ptr = asm(cap: cap) {
            aloc cap;
            hp: raw_ptr
        };

        Buffer {
            buffer: (ptr, cap, 0),
        }
    }

    pub fn append_raw(self, r: (raw_ptr, u64)) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(self.buffer, r),
        }
    }

    /// Returns the pointer to the start of the buffer's backing allocation.
    pub fn ptr(self) -> raw_ptr {
        self.buffer.0
    }

    /// Returns the number of bytes the buffer can hold without reallocating.
    pub fn capacity(self) -> u64 {
        self.buffer.1
    }

    /// Returns the number of bytes currently written to the buffer.
    pub fn len(self) -> u64 {
        self.buffer.2
    }
}

impl AsRawSlice for Buffer {
    fn as_raw_slice(self) -> raw_slice {
        __encode_buffer_as_raw_slice(self.buffer)
    }
}

pub struct BufferReader {
    pub ptr: raw_ptr,
}

impl BufferReader {
    pub fn from_parts(ptr: raw_ptr, _len: u64) -> BufferReader {
        BufferReader { ptr }
    }

    #[inline(always)]
    pub fn from_first_parameter() -> raw_ptr {
        const FIRST_PARAMETER_OFFSET: u64 = 73;

        let ptr = asm() {
            fp: raw_ptr
        };
        let ptr = ptr.add::<u64>(FIRST_PARAMETER_OFFSET);
        let ptr = ptr.read::<u64>();

        asm(ptr: ptr) {
            ptr: raw_ptr
        }
    }

    #[inline(always)]
    pub fn from_second_parameter() -> raw_ptr {
        const SECOND_PARAMETER_OFFSET: u64 = 74;

        let ptr = asm() {
            fp: raw_ptr
        };
        let ptr = ptr.add::<u64>(SECOND_PARAMETER_OFFSET);
        let ptr = ptr.read::<u64>();

        asm(ptr: ptr) {
            ptr: raw_ptr
        }
    }

    #[inline(always)]
    pub fn from_script_data() -> raw_ptr {
        __gtf::<raw_ptr>(0, 0xA)
    }

    #[inline(always)]
    pub fn from_predicate_data() -> raw_ptr {
        let predicate_index = asm(r1) {
            gm r1 i3; // GET_VERIFYING_PREDICATE
            r1: u64
        };
        Self::from_predicate_data_by_index(predicate_index)
    }

    #[inline(always)]
    pub fn from_predicate_data_by_index(predicate_index: u64) -> raw_ptr {
        match __gtf::<u8>(predicate_index, 0x200) { // GTF_INPUT_TYPE
            0u8 => __gtf::<raw_ptr>(predicate_index, 0x20C), // INPUT_COIN_PREDICATE_DATA
            2u8 => __gtf::<raw_ptr>(predicate_index, 0x24A), // INPUT_MESSAGE_PREDICATE_DATA
            _ => __revert(0),
        }
    }

    pub fn read_8_bytes<T>(ref mut self) -> T {
        let v = asm(ptr: self.ptr, val) {
            lw val ptr i0;
            val: T
        };
        self.ptr = __ptr_add::<u8>(self.ptr, 8);
        v
    }

    pub fn read_32_bytes<T>(ref mut self) -> T {
        let v = asm(ptr: self.ptr) {
            ptr: T
        };
        self.ptr = __ptr_add::<u8>(self.ptr, 32);
        v
    }

    pub fn read_bytes(ref mut self, count: u64) -> raw_slice {
        let slice = asm(ptr: (self.ptr, count)) {
            ptr: raw_slice
        };
        self.ptr = __ptr_add::<u8>(self.ptr, count);
        slice
    }

    pub fn read<T>(ref mut self) -> T {
        let size = __size_of::<T>();

        if __is_reference_type::<T>() {
            let v = asm(ptr: self.ptr) {
                ptr: T
            };
            self.ptr = __ptr_add::<u8>(self.ptr, size);
            v
        } else if size == 1 {
            let v = asm(ptr: self.ptr, val) {
                lb val ptr i0;
                val: T
            };
            self.ptr = __ptr_add::<u8>(self.ptr, 1);
            v
        } else {
            self.read_8_bytes::<T>()
        }
    }

    pub fn decode<T>(ref mut self) -> T
    where
        T: AbiDecode,
    {
        T::abi_decode(self)
    }

    pub fn ptr(self) -> raw_ptr {
        self.ptr
    }
}

// is trivial?

pub fn is_encode_trivial<T>() -> bool
where
    T: AbiEncode,
{
    T::is_encode_trivial()
}

pub fn is_decode_trivial<T>() -> bool
where
    T: AbiDecode,
{
    T::is_decode_trivial()
}

// Encode

pub trait AbiEncode {
    fn is_encode_trivial() -> bool;
    fn abi_encode(self, buffer: Buffer) -> Buffer;
}

impl AbiEncode for bool {
    fn is_encode_trivial() -> bool {
        true
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

// Encode Numbers

impl AbiEncode for b256 {
    fn is_encode_trivial() -> bool {
        true
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u256 {
    fn is_encode_trivial() -> bool {
        true
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u64 {
    fn is_encode_trivial() -> bool {
        true
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u32 {
    fn is_encode_trivial() -> bool {
        false
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u16 {
    fn is_encode_trivial() -> bool {
        false
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u8 {
    fn is_encode_trivial() -> bool {
        true
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

// Encode str slice and str arrays

impl AbiEncode for str {
    fn is_encode_trivial() -> bool {
        false
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

#[cfg(experimental_str_array_no_padding = false)]
impl<const N: u64> AbiEncode for str[N] {
    // str[N] have alignments and paddings that make them not trivial
    // for more information see comments on a test named: string_array
    fn is_encode_trivial() -> bool {
        false
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

#[cfg(experimental_str_array_no_padding = true)]
impl<const N: u64> AbiEncode for str[N] {
    fn is_encode_trivial() -> bool {
        true
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

// Encode Arrays and Slices

impl AbiEncode for raw_slice {
    fn is_encode_trivial() -> bool {
        false
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl<T, const N: u64> AbiEncode for [T; N]
where
    T: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        is_encode_trivial::<T>()
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;

        while i < N {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };

        buffer
    }
}

// Encode Tuples

impl AbiEncode for () {
    fn is_encode_trivial() -> bool {
        true
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        buffer
    }
}

// BEGIN TUPLES_ENCODE
impl<A> AbiEncode for (A, )
where
    A: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        buffer
    }
}
impl<A, B> AbiEncode for (A, B)
where
    A: AbiEncode,
    B: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C> AbiEncode for (A, B, C)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D> AbiEncode for (A, B, C, D)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E> AbiEncode for (A, B, C, D, E)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F> AbiEncode for (A, B, C, D, E, F)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G> AbiEncode for (A, B, C, D, E, F, G)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H> AbiEncode for (A, B, C, D, E, F, G, H)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I> AbiEncode for (A, B, C, D, E, F, G, H, I)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J> AbiEncode for (A, B, C, D, E, F, G, H, I, J)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
    R: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        let r = r && is_encode_trivial::<R>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        let buffer = self.17.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
    R: AbiEncode,
    S: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        let r = r && is_encode_trivial::<R>();
        let r = r && is_encode_trivial::<S>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        let buffer = self.17.abi_encode(buffer);
        let buffer = self.18.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
    R: AbiEncode,
    S: AbiEncode,
    T: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        let r = r && is_encode_trivial::<R>();
        let r = r && is_encode_trivial::<S>();
        let r = r && is_encode_trivial::<T>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        let buffer = self.17.abi_encode(buffer);
        let buffer = self.18.abi_encode(buffer);
        let buffer = self.19.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
    R: AbiEncode,
    S: AbiEncode,
    T: AbiEncode,
    U: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        let r = r && is_encode_trivial::<R>();
        let r = r && is_encode_trivial::<S>();
        let r = r && is_encode_trivial::<T>();
        let r = r && is_encode_trivial::<U>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        let buffer = self.17.abi_encode(buffer);
        let buffer = self.18.abi_encode(buffer);
        let buffer = self.19.abi_encode(buffer);
        let buffer = self.20.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
    R: AbiEncode,
    S: AbiEncode,
    T: AbiEncode,
    U: AbiEncode,
    V: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        let r = r && is_encode_trivial::<R>();
        let r = r && is_encode_trivial::<S>();
        let r = r && is_encode_trivial::<T>();
        let r = r && is_encode_trivial::<U>();
        let r = r && is_encode_trivial::<V>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        let buffer = self.17.abi_encode(buffer);
        let buffer = self.18.abi_encode(buffer);
        let buffer = self.19.abi_encode(buffer);
        let buffer = self.20.abi_encode(buffer);
        let buffer = self.21.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
    R: AbiEncode,
    S: AbiEncode,
    T: AbiEncode,
    U: AbiEncode,
    V: AbiEncode,
    W: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        let r = r && is_encode_trivial::<R>();
        let r = r && is_encode_trivial::<S>();
        let r = r && is_encode_trivial::<T>();
        let r = r && is_encode_trivial::<U>();
        let r = r && is_encode_trivial::<V>();
        let r = r && is_encode_trivial::<W>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        let buffer = self.17.abi_encode(buffer);
        let buffer = self.18.abi_encode(buffer);
        let buffer = self.19.abi_encode(buffer);
        let buffer = self.20.abi_encode(buffer);
        let buffer = self.21.abi_encode(buffer);
        let buffer = self.22.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
    R: AbiEncode,
    S: AbiEncode,
    T: AbiEncode,
    U: AbiEncode,
    V: AbiEncode,
    W: AbiEncode,
    X: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        let r = r && is_encode_trivial::<R>();
        let r = r && is_encode_trivial::<S>();
        let r = r && is_encode_trivial::<T>();
        let r = r && is_encode_trivial::<U>();
        let r = r && is_encode_trivial::<V>();
        let r = r && is_encode_trivial::<W>();
        let r = r && is_encode_trivial::<X>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        let buffer = self.17.abi_encode(buffer);
        let buffer = self.18.abi_encode(buffer);
        let buffer = self.19.abi_encode(buffer);
        let buffer = self.20.abi_encode(buffer);
        let buffer = self.21.abi_encode(buffer);
        let buffer = self.22.abi_encode(buffer);
        let buffer = self.23.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
    R: AbiEncode,
    S: AbiEncode,
    T: AbiEncode,
    U: AbiEncode,
    V: AbiEncode,
    W: AbiEncode,
    X: AbiEncode,
    Y: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        let r = r && is_encode_trivial::<R>();
        let r = r && is_encode_trivial::<S>();
        let r = r && is_encode_trivial::<T>();
        let r = r && is_encode_trivial::<U>();
        let r = r && is_encode_trivial::<V>();
        let r = r && is_encode_trivial::<W>();
        let r = r && is_encode_trivial::<X>();
        let r = r && is_encode_trivial::<Y>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        let buffer = self.17.abi_encode(buffer);
        let buffer = self.18.abi_encode(buffer);
        let buffer = self.19.abi_encode(buffer);
        let buffer = self.20.abi_encode(buffer);
        let buffer = self.21.abi_encode(buffer);
        let buffer = self.22.abi_encode(buffer);
        let buffer = self.23.abi_encode(buffer);
        let buffer = self.24.abi_encode(buffer);
        buffer
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z> AbiEncode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
    E: AbiEncode,
    F: AbiEncode,
    G: AbiEncode,
    H: AbiEncode,
    I: AbiEncode,
    J: AbiEncode,
    K: AbiEncode,
    L: AbiEncode,
    M: AbiEncode,
    N: AbiEncode,
    O: AbiEncode,
    P: AbiEncode,
    Q: AbiEncode,
    R: AbiEncode,
    S: AbiEncode,
    T: AbiEncode,
    U: AbiEncode,
    V: AbiEncode,
    W: AbiEncode,
    X: AbiEncode,
    Y: AbiEncode,
    Z: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_encode_trivial::<A>();
        let r = r && is_encode_trivial::<B>();
        let r = r && is_encode_trivial::<C>();
        let r = r && is_encode_trivial::<D>();
        let r = r && is_encode_trivial::<E>();
        let r = r && is_encode_trivial::<F>();
        let r = r && is_encode_trivial::<G>();
        let r = r && is_encode_trivial::<H>();
        let r = r && is_encode_trivial::<I>();
        let r = r && is_encode_trivial::<J>();
        let r = r && is_encode_trivial::<K>();
        let r = r && is_encode_trivial::<L>();
        let r = r && is_encode_trivial::<M>();
        let r = r && is_encode_trivial::<N>();
        let r = r && is_encode_trivial::<O>();
        let r = r && is_encode_trivial::<P>();
        let r = r && is_encode_trivial::<Q>();
        let r = r && is_encode_trivial::<R>();
        let r = r && is_encode_trivial::<S>();
        let r = r && is_encode_trivial::<T>();
        let r = r && is_encode_trivial::<U>();
        let r = r && is_encode_trivial::<V>();
        let r = r && is_encode_trivial::<W>();
        let r = r && is_encode_trivial::<X>();
        let r = r && is_encode_trivial::<Y>();
        let r = r && is_encode_trivial::<Z>();
        r
    }
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let buffer = self.0.abi_encode(buffer);
        let buffer = self.1.abi_encode(buffer);
        let buffer = self.2.abi_encode(buffer);
        let buffer = self.3.abi_encode(buffer);
        let buffer = self.4.abi_encode(buffer);
        let buffer = self.5.abi_encode(buffer);
        let buffer = self.6.abi_encode(buffer);
        let buffer = self.7.abi_encode(buffer);
        let buffer = self.8.abi_encode(buffer);
        let buffer = self.9.abi_encode(buffer);
        let buffer = self.10.abi_encode(buffer);
        let buffer = self.11.abi_encode(buffer);
        let buffer = self.12.abi_encode(buffer);
        let buffer = self.13.abi_encode(buffer);
        let buffer = self.14.abi_encode(buffer);
        let buffer = self.15.abi_encode(buffer);
        let buffer = self.16.abi_encode(buffer);
        let buffer = self.17.abi_encode(buffer);
        let buffer = self.18.abi_encode(buffer);
        let buffer = self.19.abi_encode(buffer);
        let buffer = self.20.abi_encode(buffer);
        let buffer = self.21.abi_encode(buffer);
        let buffer = self.22.abi_encode(buffer);
        let buffer = self.23.abi_encode(buffer);
        let buffer = self.24.abi_encode(buffer);
        let buffer = self.25.abi_encode(buffer);
        buffer
    }
}
// END TUPLES_ENCODE

pub fn encode<T>(item: T) -> raw_slice
where
    T: AbiEncode,
{
    const IS_TRIVIAL: bool = is_encode_trivial::<T>();
    if IS_TRIVIAL {
        let size = __size_of::<T>();
        let ptr = asm(size: size, src: &item) {
            aloc size;
            mcp hp src size;
            hp: raw_ptr
        };
        asm(s: (ptr, size)) {
            s: raw_slice
        }
    } else {
        let buffer = item.abi_encode(Buffer::new());
        buffer.as_raw_slice()
    }
}

pub fn encode_allow_alias<T>(item: &T) -> raw_slice
where
    T: AbiEncode,
{
    if is_encode_trivial::<T>() {
        let size = __size_of::<T>();
        __transmute::<(&T, u64), raw_slice>((item, size))
    } else {
        let buffer = (*item).abi_encode(Buffer::new());
        buffer.as_raw_slice()
    }
}

pub fn encode_and_return<T>(item: &T) -> !
where
    T: AbiEncode,
{
    const IS_TRIVIAL: bool = is_encode_trivial::<T>();
    if IS_TRIVIAL {
        let size = __size_of::<T>();
        __contract_ret(item, size);
    } else {
        let item = *item;
        let buffer = item.abi_encode(Buffer::new());
        __contract_ret(buffer.buffer.0, buffer.buffer.2);
    }
}

pub fn encode_configurable<T>(item: T) -> raw_slice
where
    T: AbiEncode,
{
    let buffer = item.abi_encode(Buffer::new());
    buffer.as_raw_slice()
}

pub fn abi_decode<T>(data: raw_slice) -> T
where
    T: AbiDecode,
{
    if is_decode_trivial::<T>() {
        let size = __size_of::<T>();
        let item: &T = asm(size: size, src: data.ptr()) {
            aloc size;
            mcp hp src size;
            hp: &T
        };
        *item
    } else {
        let mut buffer = BufferReader::from_parts(data.ptr(), data.len::<u8>());
        T::abi_decode(buffer)
    }
}

pub fn abi_decode_in_place<T>(ptr: raw_ptr, len: u64, target: raw_ptr)
where
    T: AbiDecode,
{
    if is_decode_trivial::<T>() {
        asm(src: ptr, target: target, len: len) {
            mcp target src len;
        }
    } else {
        let mut buffer = BufferReader::from_parts(ptr, len);
        let temp = T::abi_decode(buffer);
        asm(
            target: target,
            temp: __addr_of(temp),
            size: __size_of::<T>(),
        ) {
            mcp target temp size;
        }
    }
}

// Decode

pub trait AbiDecode {
    fn is_decode_trivial() -> bool;
    fn abi_decode(ref mut buffer: BufferReader) -> Self;
}

impl AbiDecode for b256 {
    fn is_decode_trivial() -> bool {
        true
    }
    fn abi_decode(ref mut buffer: BufferReader) -> b256 {
        buffer.read_32_bytes::<b256>()
    }
}

impl AbiDecode for u256 {
    fn is_decode_trivial() -> bool {
        true
    }
    fn abi_decode(ref mut buffer: BufferReader) -> u256 {
        buffer.read_32_bytes::<u256>()
    }
}

impl AbiDecode for u64 {
    fn is_decode_trivial() -> bool {
        true
    }
    fn abi_decode(ref mut buffer: BufferReader) -> u64 {
        buffer.read_8_bytes::<u64>()
    }
}

pub fn as_u16(input: u8) -> u16 {
    asm(input: input) {
        input: u16
    }
}

pub fn as_u32(input: u8) -> u32 {
    asm(input: input) {
        input: u32
    }
}

impl AbiDecode for u32 {
    fn is_decode_trivial() -> bool {
        false
    }
    fn abi_decode(ref mut buffer: BufferReader) -> u32 {
        let a = as_u32(buffer.read::<u8>());
        let b = as_u32(buffer.read::<u8>());
        let c = as_u32(buffer.read::<u8>());
        let d = as_u32(buffer.read::<u8>());
        (a << 24) | (b << 16) | (c << 8) | d
    }
}

impl AbiDecode for u16 {
    fn is_decode_trivial() -> bool {
        false
    }
    fn abi_decode(ref mut buffer: BufferReader) -> u16 {
        let a = as_u16(buffer.read::<u8>());
        let b = as_u16(buffer.read::<u8>());
        (a << 8) | b
    }
}

impl AbiDecode for u8 {
    fn is_decode_trivial() -> bool {
        true
    }
    fn abi_decode(ref mut buffer: BufferReader) -> u8 {
        buffer.read::<u8>()
    }
}

impl AbiDecode for bool {
    fn is_decode_trivial() -> bool {
        false
    }
    fn abi_decode(ref mut buffer: BufferReader) -> bool {
        match buffer.read::<u8>() {
            0 => false,
            1 => true,
            _ => __revert(0),
        }
    }
}

impl AbiDecode for raw_slice {
    fn is_decode_trivial() -> bool {
        false
    }
    fn abi_decode(ref mut buffer: BufferReader) -> raw_slice {
        let len = buffer.read_8_bytes::<u64>();
        buffer.read_bytes(len)
    }
}

impl AbiDecode for str {
    fn is_decode_trivial() -> bool {
        false
    }
    fn abi_decode(ref mut buffer: BufferReader) -> str {
        let len = buffer.read_8_bytes::<u64>();
        let data = buffer.read_bytes(len);
        asm(s: (data.ptr(), len)) {
            s: str
        }
    }
}

#[cfg(experimental_str_array_no_padding = false)]
impl<const N: u64> AbiDecode for str[N] {
    // see comments on `is_encode_trivial` for str[N] above
    fn is_decode_trivial() -> bool {
        false
    }
    fn abi_decode(ref mut buffer: BufferReader) -> str[N] {
        let data = buffer.read_bytes(N);
        asm(s: data.ptr()) {
            s: str[N]
        }
    }
}

#[cfg(experimental_str_array_no_padding = true)]
impl<const N: u64> AbiDecode for str[N] {
    fn is_decode_trivial() -> bool {
        true
    }
    fn abi_decode(ref mut buffer: BufferReader) -> str[N] {
        let data = buffer.read_bytes(N);
        asm(s: data.ptr()) {
            s: str[N]
        }
    }
}

impl<T, const N: u64> AbiDecode for [T; N]
where
    T: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        is_decode_trivial::<T>()
    }
    fn abi_decode(ref mut buffer: BufferReader) -> [T; N] {
        const LENGTH: u64 = __size_of::<T>() * N;
        let mut array = [0u8; LENGTH];
        let array: &mut [T; N] = __transmute::<&mut [u8; LENGTH], &mut [T; N]>(&mut array);

        let mut i = 0;

        while i < N {
            let item: &mut T = __elem_at(array, i);
            *item = buffer.decode::<T>();
            i += 1;
        }

        *array
    }
}

impl AbiDecode for () {
    fn is_decode_trivial() -> bool {
        true
    }
    fn abi_decode(ref mut _buffer: BufferReader) -> () {
        ()
    }
}

// BEGIN TUPLES_DECODE
impl<A> AbiDecode for (A, )
where
    A: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (A::abi_decode(buffer), )
    }
}
impl<A, B> AbiDecode for (A, B)
where
    A: AbiDecode,
    B: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (A::abi_decode(buffer), B::abi_decode(buffer))
    }
}
impl<A, B, C> AbiDecode for (A, B, C)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (A::abi_decode(buffer), B::abi_decode(buffer), C::abi_decode(buffer))
    }
}
impl<A, B, C, D> AbiDecode for (A, B, C, D)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E> AbiDecode for (A, B, C, D, E)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F> AbiDecode for (A, B, C, D, E, F)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G> AbiDecode for (A, B, C, D, E, F, G)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H> AbiDecode for (A, B, C, D, E, F, G, H)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I> AbiDecode for (A, B, C, D, E, F, G, H, I)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J> AbiDecode for (A, B, C, D, E, F, G, H, I, J)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
    R: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        let r = r && is_decode_trivial::<R>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
            R::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
    R: AbiDecode,
    S: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        let r = r && is_decode_trivial::<R>();
        let r = r && is_decode_trivial::<S>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
            R::abi_decode(buffer),
            S::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
    R: AbiDecode,
    S: AbiDecode,
    T: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        let r = r && is_decode_trivial::<R>();
        let r = r && is_decode_trivial::<S>();
        let r = r && is_decode_trivial::<T>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
            R::abi_decode(buffer),
            S::abi_decode(buffer),
            T::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
    R: AbiDecode,
    S: AbiDecode,
    T: AbiDecode,
    U: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        let r = r && is_decode_trivial::<R>();
        let r = r && is_decode_trivial::<S>();
        let r = r && is_decode_trivial::<T>();
        let r = r && is_decode_trivial::<U>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
            R::abi_decode(buffer),
            S::abi_decode(buffer),
            T::abi_decode(buffer),
            U::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
    R: AbiDecode,
    S: AbiDecode,
    T: AbiDecode,
    U: AbiDecode,
    V: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        let r = r && is_decode_trivial::<R>();
        let r = r && is_decode_trivial::<S>();
        let r = r && is_decode_trivial::<T>();
        let r = r && is_decode_trivial::<U>();
        let r = r && is_decode_trivial::<V>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
            R::abi_decode(buffer),
            S::abi_decode(buffer),
            T::abi_decode(buffer),
            U::abi_decode(buffer),
            V::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
    R: AbiDecode,
    S: AbiDecode,
    T: AbiDecode,
    U: AbiDecode,
    V: AbiDecode,
    W: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        let r = r && is_decode_trivial::<R>();
        let r = r && is_decode_trivial::<S>();
        let r = r && is_decode_trivial::<T>();
        let r = r && is_decode_trivial::<U>();
        let r = r && is_decode_trivial::<V>();
        let r = r && is_decode_trivial::<W>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
            R::abi_decode(buffer),
            S::abi_decode(buffer),
            T::abi_decode(buffer),
            U::abi_decode(buffer),
            V::abi_decode(buffer),
            W::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
    R: AbiDecode,
    S: AbiDecode,
    T: AbiDecode,
    U: AbiDecode,
    V: AbiDecode,
    W: AbiDecode,
    X: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        let r = r && is_decode_trivial::<R>();
        let r = r && is_decode_trivial::<S>();
        let r = r && is_decode_trivial::<T>();
        let r = r && is_decode_trivial::<U>();
        let r = r && is_decode_trivial::<V>();
        let r = r && is_decode_trivial::<W>();
        let r = r && is_decode_trivial::<X>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
            R::abi_decode(buffer),
            S::abi_decode(buffer),
            T::abi_decode(buffer),
            U::abi_decode(buffer),
            V::abi_decode(buffer),
            W::abi_decode(buffer),
            X::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
    R: AbiDecode,
    S: AbiDecode,
    T: AbiDecode,
    U: AbiDecode,
    V: AbiDecode,
    W: AbiDecode,
    X: AbiDecode,
    Y: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        let r = r && is_decode_trivial::<R>();
        let r = r && is_decode_trivial::<S>();
        let r = r && is_decode_trivial::<T>();
        let r = r && is_decode_trivial::<U>();
        let r = r && is_decode_trivial::<V>();
        let r = r && is_decode_trivial::<W>();
        let r = r && is_decode_trivial::<X>();
        let r = r && is_decode_trivial::<Y>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
            R::abi_decode(buffer),
            S::abi_decode(buffer),
            T::abi_decode(buffer),
            U::abi_decode(buffer),
            V::abi_decode(buffer),
            W::abi_decode(buffer),
            X::abi_decode(buffer),
            Y::abi_decode(buffer),
        )
    }
}
impl<A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z> AbiDecode for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z)
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
    E: AbiDecode,
    F: AbiDecode,
    G: AbiDecode,
    H: AbiDecode,
    I: AbiDecode,
    J: AbiDecode,
    K: AbiDecode,
    L: AbiDecode,
    M: AbiDecode,
    N: AbiDecode,
    O: AbiDecode,
    P: AbiDecode,
    Q: AbiDecode,
    R: AbiDecode,
    S: AbiDecode,
    T: AbiDecode,
    U: AbiDecode,
    V: AbiDecode,
    W: AbiDecode,
    X: AbiDecode,
    Y: AbiDecode,
    Z: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        let r = __runtime_mem_id::<Self>() == __encoding_mem_id::<Self>();
        let r = r && is_decode_trivial::<A>();
        let r = r && is_decode_trivial::<B>();
        let r = r && is_decode_trivial::<C>();
        let r = r && is_decode_trivial::<D>();
        let r = r && is_decode_trivial::<E>();
        let r = r && is_decode_trivial::<F>();
        let r = r && is_decode_trivial::<G>();
        let r = r && is_decode_trivial::<H>();
        let r = r && is_decode_trivial::<I>();
        let r = r && is_decode_trivial::<J>();
        let r = r && is_decode_trivial::<K>();
        let r = r && is_decode_trivial::<L>();
        let r = r && is_decode_trivial::<M>();
        let r = r && is_decode_trivial::<N>();
        let r = r && is_decode_trivial::<O>();
        let r = r && is_decode_trivial::<P>();
        let r = r && is_decode_trivial::<Q>();
        let r = r && is_decode_trivial::<R>();
        let r = r && is_decode_trivial::<S>();
        let r = r && is_decode_trivial::<T>();
        let r = r && is_decode_trivial::<U>();
        let r = r && is_decode_trivial::<V>();
        let r = r && is_decode_trivial::<W>();
        let r = r && is_decode_trivial::<X>();
        let r = r && is_decode_trivial::<Y>();
        let r = r && is_decode_trivial::<Z>();
        r
    }
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (
            A::abi_decode(buffer),
            B::abi_decode(buffer),
            C::abi_decode(buffer),
            D::abi_decode(buffer),
            E::abi_decode(buffer),
            F::abi_decode(buffer),
            G::abi_decode(buffer),
            H::abi_decode(buffer),
            I::abi_decode(buffer),
            J::abi_decode(buffer),
            K::abi_decode(buffer),
            L::abi_decode(buffer),
            M::abi_decode(buffer),
            N::abi_decode(buffer),
            O::abi_decode(buffer),
            P::abi_decode(buffer),
            Q::abi_decode(buffer),
            R::abi_decode(buffer),
            S::abi_decode(buffer),
            T::abi_decode(buffer),
            U::abi_decode(buffer),
            V::abi_decode(buffer),
            W::abi_decode(buffer),
            X::abi_decode(buffer),
            Y::abi_decode(buffer),
            Z::abi_decode(buffer),
        )
    }
}
// END TUPLES_DECODE
use ::ops::*;

pub fn contract_call<T, TArgs>(
    contract_id: b256,
    method_name: raw_slice,
    args: TArgs,
    coins: u64,
    asset_id: b256,
    gas: u64,
) -> T
where
    T: AbiDecode,
    TArgs: AbiEncode,
{
    let second_parameter = encode(args);
    let params = (
        contract_id,
        asm(a: method_name.ptr()) {
            a: u64
        },
        asm(a: second_parameter.ptr()) {
            a: u64
        },
    );

    __contract_call(&params, coins, asset_id, gas);
    let ptr = asm() {
        ret: raw_ptr
    };

    decode_from_raw_ptr::<T>(ptr)
}

#[inline(always)]
pub fn decode_from_raw_ptr<T>(ptr: raw_ptr) -> T
where
    T: AbiDecode,
{
    if is_decode_trivial::<T>() {
        let ptr: &T = __transmute::<raw_ptr, &T>(ptr);
        *ptr
    } else {
        let mut buffer = BufferReader { ptr };
        T::abi_decode(buffer)
    }
}

pub fn decode_script_data<T>() -> T
where
    T: AbiDecode,
{
    decode_from_raw_ptr::<T>(BufferReader::from_script_data())
}

pub fn decode_predicate_data<T>() -> T
where
    T: AbiDecode,
{
    decode_from_raw_ptr::<T>(BufferReader::from_predicate_data())
}

pub fn decode_predicate_data_by_index<T>(index: u64) -> T
where
    T: AbiDecode,
{
    decode_from_raw_ptr::<T>(BufferReader::from_predicate_data_by_index(index))
}

pub fn decode_first_param<T>() -> T
where
    T: AbiDecode,
{
    decode_from_raw_ptr::<T>(BufferReader::from_first_parameter())
}

pub fn decode_second_param<T>() -> T
where
    T: AbiDecode,
{
    decode_from_raw_ptr::<T>(BufferReader::from_second_parameter())
}

pub struct TrivialBool {
    value: u64,
}

impl AbiEncode for TrivialBool {
    fn is_encode_trivial() -> bool {
        true
    }

    fn abi_encode(self, buffer: Buffer) -> Buffer {
        self.value.abi_encode(buffer)
    }
}

impl AbiDecode for TrivialBool {
    fn is_decode_trivial() -> bool {
        true
    }

    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        let value = u64::abi_decode(buffer);
        TrivialBool { value }
    }
}

impl TrivialBool {
    pub fn from(value: bool) -> Self {
        TrivialBool {
            value: if value { 1 } else { 0 },
        }
    }

    pub fn is_valid(self) -> bool {
        match self.value {
            0 => true,
            1 => true,
            _ => false,
        }
    }

    pub fn unwrap(self) -> bool {
        match self.value {
            0 => false,
            1 => true,
            _ => __revert(REVERT_WITH_TRIVIAL_BOOL_UNWRAP),
        }
    }
}

pub struct TrivialEnum<T> {
    value: T,
}

impl<T> TrivialEnum<T> {
    pub fn from(value: T) -> TrivialEnum<T> {
        TrivialEnum { value }
    }
}

pub trait EnumCodecValues {
    fn is_decode_trivial_table() -> &__slice[bool];
}

impl<T> TrivialEnum<T>
where
    T: EnumCodecValues,
{
    pub fn is_valid(self) -> bool {
        let discriminant: raw_slice = raw_slice::from_parts::<u8>(__addr_of(self.value), 8);
        let discriminant: u64 = abi_decode::<u64>(discriminant);

        let is_decode_trivial_table = T::is_decode_trivial_table();

        if discriminant < is_decode_trivial_table.len() {
            *__elem_at(is_decode_trivial_table, discriminant)
        } else {
            false
        }
    }

    pub fn unwrap(self) -> T {
        if self.is_valid() {
            self.value
        } else {
            __revert(REVERT_WITH_TRIVIAL_ENUM_UNWRAP)
        }
    }
}

impl<T> AbiEncode for TrivialEnum<T>
where
    T: AbiEncode,
{
    fn is_encode_trivial() -> bool {
        true
    }

    fn abi_encode(self, buffer: Buffer) -> Buffer {
        self.value.abi_encode(buffer)
    }
}

impl<T> AbiDecode for TrivialEnum<T>
where
    T: AbiDecode,
{
    fn is_decode_trivial() -> bool {
        true
    }

    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        let value = T::abi_decode(buffer);
        TrivialEnum { value }
    }
}
