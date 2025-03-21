library;

use ::ops::*;
use ::raw_ptr::*;
use ::raw_slice::*;

pub struct Buffer {
    buffer: (raw_ptr, u64, u64), // ptr, capacity, size
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buffer: __encode_buffer_empty(),
        }
    }

    fn with_capacity(cap: u64) -> Self {
        let ptr = asm(cap: cap) {
            aloc cap;
            hp: raw_ptr
        };

        Buffer {
            buffer: (ptr, cap, 0),
        }
    }
}

impl AsRawSlice for Buffer {
    fn as_raw_slice(self) -> raw_slice {
        __encode_buffer_as_raw_slice(self.buffer)
    }
}

pub struct BufferReader {
    ptr: raw_ptr,
}

impl BufferReader {
    pub fn from_parts(ptr: raw_ptr, _len: u64) -> BufferReader {
        BufferReader { ptr }
    }

    pub fn from_first_parameter() -> BufferReader {
        const FIRST_PARAMETER_OFFSET: u64 = 73;

        let ptr = asm() {
            fp: raw_ptr
        };
        let ptr = ptr.add::<u64>(FIRST_PARAMETER_OFFSET);
        let ptr = ptr.read::<u64>();

        BufferReader {
            ptr: asm(ptr: ptr) {
                ptr: raw_ptr
            },
        }
    }

    pub fn from_second_parameter() -> BufferReader {
        const SECOND_PARAMETER_OFFSET: u64 = 74;

        let ptr = asm() {
            fp: raw_ptr
        };
        let ptr = ptr.add::<u64>(SECOND_PARAMETER_OFFSET);
        let ptr = ptr.read::<u64>();

        BufferReader {
            ptr: asm(ptr: ptr) {
                ptr: raw_ptr
            },
        }
    }

    pub fn from_script_data() -> BufferReader {
        let ptr = __gtf::<raw_ptr>(0, 0xA); // SCRIPT_DATA
        let _len = __gtf::<u64>(0, 0x4); // SCRIPT_DATA_LEN
        BufferReader { ptr }
    }

    pub fn from_predicate_data() -> BufferReader {
        let predicate_index = asm(r1) {
            gm r1 i3; // GET_VERIFYING_PREDICATE
            r1: u64
        };
        Self::from_predicate_data_by_index(predicate_index)
    }

    pub fn from_predicate_data_by_index(predicate_index: u64) -> BufferReader {
        match __gtf::<u8>(predicate_index, 0x200) { // GTF_INPUT_TYPE
            0u8 => {
                let ptr = __gtf::<raw_ptr>(predicate_index, 0x20C); // INPUT_COIN_PREDICATE_DATA
                let _len = __gtf::<u64>(predicate_index, 0x20A); // INPUT_COIN_PREDICATE_DATA_LENGTH
                BufferReader { ptr }
            },
            2u8 => {
                let ptr = __gtf::<raw_ptr>(predicate_index, 0x24A); // INPUT_MESSAGE_PREDICATE_DATA
                let _len = __gtf::<u64>(predicate_index, 0x247); // INPUT_MESSAGE_PREDICATE_DATA_LENGTH
                BufferReader { ptr }
            },
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
}

// Encode

pub trait AbiEncode {
    fn abi_encode(self, buffer: Buffer) -> Buffer;
}

impl AbiEncode for bool {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

// Encode Numbers

impl AbiEncode for b256 {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u256 {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u64 {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u32 {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u16 {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for u8 {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

// Encode str slice for raw ptr

impl AbiEncode for raw_ptr {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let v = asm(p: self) {
            p: u64
        };
        v.abi_encode(buffer)
    }
}

// Encode str slice and str arrays

impl AbiEncode for str {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

impl AbiEncode for str[0] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        buffer
    }
}

// BEGIN STRARRAY_ENCODE
impl AbiEncode for str[1] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[2] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[3] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[4] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[5] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[6] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[7] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[8] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[9] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[10] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[11] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[12] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[13] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[14] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[15] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[16] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[17] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[18] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[19] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[20] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[21] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[22] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[23] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[24] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[25] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[26] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[27] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[28] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[29] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[30] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[31] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[32] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[33] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[34] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[35] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[36] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[37] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[38] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[39] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[40] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[41] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[42] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[43] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[44] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[45] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[46] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[47] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[48] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[49] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[50] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[51] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[52] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[53] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[54] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[55] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[56] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[57] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[58] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[59] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[60] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[61] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[62] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[63] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
impl AbiEncode for str[64] {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}
// END STRARRAY_ENCODE

// Encode Arrays and Slices

impl AbiEncode for raw_slice {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        Buffer {
            buffer: __encode_buffer_append(buffer.buffer, self),
        }
    }
}

#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> AbiEncode for [T; N]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < N {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        }
        buffer
    }
}

impl<T> AbiEncode for &[T]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let len = self.len();
        let mut buffer = len.abi_encode(buffer);

        let mut i = 0;
        while i < len {
            let elem = *__elem_at(self, i);
            buffer = elem.abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}

#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 0]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        buffer
    }
}

// BEGIN ARRAY_ENCODE
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 1]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 1 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 2]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 2 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 3]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 3 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 4]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 4 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 5]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 5 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 6]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 6 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 7]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 7 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 8]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 8 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 9]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 9 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 10]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 10 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 11]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 11 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 12]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 12 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 13]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 13 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 14]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 14 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 15]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 15 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 16]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 16 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 17]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 17 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 18]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 18 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 19]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 19 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 20]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 20 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 21]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 21 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 22]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 22 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 23]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 23 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 24]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 24 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 25]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 25 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 26]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 26 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 27]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 27 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 28]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 28 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 29]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 29 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 30]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 30 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 31]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 31 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 32]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 32 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 33]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 33 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 34]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 34 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 35]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 35 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 36]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 36 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 37]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 37 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 38]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 38 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 39]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 39 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 40]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 40 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 41]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 41 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 42]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 42 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 43]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 43 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 44]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 44 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 45]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 45 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 46]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 46 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 47]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 47 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 48]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 48 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 49]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 49 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 50]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 50 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 51]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 51 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 52]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 52 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 53]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 53 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 54]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 54 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 55]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 55 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 56]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 56 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 57]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 57 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 58]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 58 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 59]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 59 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 60]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 60 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 61]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 61 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 62]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 62 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 63]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 63 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiEncode for [T; 64]
where
    T: AbiEncode,
{
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let mut buffer = buffer;
        let mut i = 0;
        while i < 64 {
            buffer = self[i].abi_encode(buffer);
            i += 1;
        };
        buffer
    }
}
// END ARRAY_ENCODE

// Encode Tuples

impl AbiEncode for () {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        buffer
    }
}

// BEGIN TUPLES_ENCODE
impl<A> AbiEncode for (A, )
where
    A: AbiEncode,
{
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
    let buffer = item.abi_encode(Buffer::new());
    buffer.as_raw_slice()
}

#[inline(never)]
pub fn abi_decode<T>(data: raw_slice) -> T
where
    T: AbiDecode,
{
    let mut buffer = BufferReader::from_parts(data.ptr(), data.len::<u8>());
    T::abi_decode(buffer)
}

#[inline(never)]
pub fn abi_decode_in_place<T>(ptr: raw_ptr, len: u64, target: raw_ptr)
where
    T: AbiDecode,
{
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

// Decode

pub trait AbiDecode {
    fn abi_decode(ref mut buffer: BufferReader) -> Self;
}

impl AbiDecode for b256 {
    fn abi_decode(ref mut buffer: BufferReader) -> b256 {
        buffer.read_32_bytes::<b256>()
    }
}

impl AbiDecode for u256 {
    fn abi_decode(ref mut buffer: BufferReader) -> u256 {
        buffer.read_32_bytes::<u256>()
    }
}

impl AbiDecode for u64 {
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
    fn abi_decode(ref mut buffer: BufferReader) -> u32 {
        let a = as_u32(buffer.read::<u8>());
        let b = as_u32(buffer.read::<u8>());
        let c = as_u32(buffer.read::<u8>());
        let d = as_u32(buffer.read::<u8>());
        (a << 24) | (b << 16) | (c << 8) | d
    }
}

impl AbiDecode for u16 {
    fn abi_decode(ref mut buffer: BufferReader) -> u16 {
        let a = as_u16(buffer.read::<u8>());
        let b = as_u16(buffer.read::<u8>());
        (a << 8) | b
    }
}

impl AbiDecode for u8 {
    fn abi_decode(ref mut buffer: BufferReader) -> u8 {
        buffer.read::<u8>()
    }
}

impl AbiDecode for bool {
    fn abi_decode(ref mut buffer: BufferReader) -> bool {
        match buffer.read::<u8>() {
            0 => false,
            1 => true,
            _ => __revert(0),
        }
    }
}

impl AbiDecode for raw_slice {
    fn abi_decode(ref mut buffer: BufferReader) -> raw_slice {
        let len = buffer.read_8_bytes::<u64>();
        buffer.read_bytes(len)
    }
}

impl AbiDecode for str {
    fn abi_decode(ref mut buffer: BufferReader) -> str {
        let len = buffer.read_8_bytes::<u64>();
        let data = buffer.read_bytes(len);
        asm(s: (data.ptr(), len)) {
            s: str
        }
    }
}

// BEGIN STRARRAY_DECODE
impl AbiDecode for str[1] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[1] {
        let data = buffer.read_bytes(1);
        asm(s: data.ptr()) {
            s: str[1]
        }
    }
}
impl AbiDecode for str[2] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[2] {
        let data = buffer.read_bytes(2);
        asm(s: data.ptr()) {
            s: str[2]
        }
    }
}
impl AbiDecode for str[3] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[3] {
        let data = buffer.read_bytes(3);
        asm(s: data.ptr()) {
            s: str[3]
        }
    }
}
impl AbiDecode for str[4] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[4] {
        let data = buffer.read_bytes(4);
        asm(s: data.ptr()) {
            s: str[4]
        }
    }
}
impl AbiDecode for str[5] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[5] {
        let data = buffer.read_bytes(5);
        asm(s: data.ptr()) {
            s: str[5]
        }
    }
}
impl AbiDecode for str[6] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[6] {
        let data = buffer.read_bytes(6);
        asm(s: data.ptr()) {
            s: str[6]
        }
    }
}
impl AbiDecode for str[7] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[7] {
        let data = buffer.read_bytes(7);
        asm(s: data.ptr()) {
            s: str[7]
        }
    }
}
impl AbiDecode for str[8] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[8] {
        let data = buffer.read_bytes(8);
        asm(s: data.ptr()) {
            s: str[8]
        }
    }
}
impl AbiDecode for str[9] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[9] {
        let data = buffer.read_bytes(9);
        asm(s: data.ptr()) {
            s: str[9]
        }
    }
}
impl AbiDecode for str[10] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[10] {
        let data = buffer.read_bytes(10);
        asm(s: data.ptr()) {
            s: str[10]
        }
    }
}
impl AbiDecode for str[11] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[11] {
        let data = buffer.read_bytes(11);
        asm(s: data.ptr()) {
            s: str[11]
        }
    }
}
impl AbiDecode for str[12] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[12] {
        let data = buffer.read_bytes(12);
        asm(s: data.ptr()) {
            s: str[12]
        }
    }
}
impl AbiDecode for str[13] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[13] {
        let data = buffer.read_bytes(13);
        asm(s: data.ptr()) {
            s: str[13]
        }
    }
}
impl AbiDecode for str[14] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[14] {
        let data = buffer.read_bytes(14);
        asm(s: data.ptr()) {
            s: str[14]
        }
    }
}
impl AbiDecode for str[15] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[15] {
        let data = buffer.read_bytes(15);
        asm(s: data.ptr()) {
            s: str[15]
        }
    }
}
impl AbiDecode for str[16] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[16] {
        let data = buffer.read_bytes(16);
        asm(s: data.ptr()) {
            s: str[16]
        }
    }
}
impl AbiDecode for str[17] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[17] {
        let data = buffer.read_bytes(17);
        asm(s: data.ptr()) {
            s: str[17]
        }
    }
}
impl AbiDecode for str[18] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[18] {
        let data = buffer.read_bytes(18);
        asm(s: data.ptr()) {
            s: str[18]
        }
    }
}
impl AbiDecode for str[19] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[19] {
        let data = buffer.read_bytes(19);
        asm(s: data.ptr()) {
            s: str[19]
        }
    }
}
impl AbiDecode for str[20] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[20] {
        let data = buffer.read_bytes(20);
        asm(s: data.ptr()) {
            s: str[20]
        }
    }
}
impl AbiDecode for str[21] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[21] {
        let data = buffer.read_bytes(21);
        asm(s: data.ptr()) {
            s: str[21]
        }
    }
}
impl AbiDecode for str[22] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[22] {
        let data = buffer.read_bytes(22);
        asm(s: data.ptr()) {
            s: str[22]
        }
    }
}
impl AbiDecode for str[23] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[23] {
        let data = buffer.read_bytes(23);
        asm(s: data.ptr()) {
            s: str[23]
        }
    }
}
impl AbiDecode for str[24] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[24] {
        let data = buffer.read_bytes(24);
        asm(s: data.ptr()) {
            s: str[24]
        }
    }
}
impl AbiDecode for str[25] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[25] {
        let data = buffer.read_bytes(25);
        asm(s: data.ptr()) {
            s: str[25]
        }
    }
}
impl AbiDecode for str[26] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[26] {
        let data = buffer.read_bytes(26);
        asm(s: data.ptr()) {
            s: str[26]
        }
    }
}
impl AbiDecode for str[27] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[27] {
        let data = buffer.read_bytes(27);
        asm(s: data.ptr()) {
            s: str[27]
        }
    }
}
impl AbiDecode for str[28] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[28] {
        let data = buffer.read_bytes(28);
        asm(s: data.ptr()) {
            s: str[28]
        }
    }
}
impl AbiDecode for str[29] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[29] {
        let data = buffer.read_bytes(29);
        asm(s: data.ptr()) {
            s: str[29]
        }
    }
}
impl AbiDecode for str[30] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[30] {
        let data = buffer.read_bytes(30);
        asm(s: data.ptr()) {
            s: str[30]
        }
    }
}
impl AbiDecode for str[31] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[31] {
        let data = buffer.read_bytes(31);
        asm(s: data.ptr()) {
            s: str[31]
        }
    }
}
impl AbiDecode for str[32] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[32] {
        let data = buffer.read_bytes(32);
        asm(s: data.ptr()) {
            s: str[32]
        }
    }
}
impl AbiDecode for str[33] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[33] {
        let data = buffer.read_bytes(33);
        asm(s: data.ptr()) {
            s: str[33]
        }
    }
}
impl AbiDecode for str[34] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[34] {
        let data = buffer.read_bytes(34);
        asm(s: data.ptr()) {
            s: str[34]
        }
    }
}
impl AbiDecode for str[35] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[35] {
        let data = buffer.read_bytes(35);
        asm(s: data.ptr()) {
            s: str[35]
        }
    }
}
impl AbiDecode for str[36] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[36] {
        let data = buffer.read_bytes(36);
        asm(s: data.ptr()) {
            s: str[36]
        }
    }
}
impl AbiDecode for str[37] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[37] {
        let data = buffer.read_bytes(37);
        asm(s: data.ptr()) {
            s: str[37]
        }
    }
}
impl AbiDecode for str[38] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[38] {
        let data = buffer.read_bytes(38);
        asm(s: data.ptr()) {
            s: str[38]
        }
    }
}
impl AbiDecode for str[39] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[39] {
        let data = buffer.read_bytes(39);
        asm(s: data.ptr()) {
            s: str[39]
        }
    }
}
impl AbiDecode for str[40] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[40] {
        let data = buffer.read_bytes(40);
        asm(s: data.ptr()) {
            s: str[40]
        }
    }
}
impl AbiDecode for str[41] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[41] {
        let data = buffer.read_bytes(41);
        asm(s: data.ptr()) {
            s: str[41]
        }
    }
}
impl AbiDecode for str[42] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[42] {
        let data = buffer.read_bytes(42);
        asm(s: data.ptr()) {
            s: str[42]
        }
    }
}
impl AbiDecode for str[43] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[43] {
        let data = buffer.read_bytes(43);
        asm(s: data.ptr()) {
            s: str[43]
        }
    }
}
impl AbiDecode for str[44] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[44] {
        let data = buffer.read_bytes(44);
        asm(s: data.ptr()) {
            s: str[44]
        }
    }
}
impl AbiDecode for str[45] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[45] {
        let data = buffer.read_bytes(45);
        asm(s: data.ptr()) {
            s: str[45]
        }
    }
}
impl AbiDecode for str[46] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[46] {
        let data = buffer.read_bytes(46);
        asm(s: data.ptr()) {
            s: str[46]
        }
    }
}
impl AbiDecode for str[47] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[47] {
        let data = buffer.read_bytes(47);
        asm(s: data.ptr()) {
            s: str[47]
        }
    }
}
impl AbiDecode for str[48] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[48] {
        let data = buffer.read_bytes(48);
        asm(s: data.ptr()) {
            s: str[48]
        }
    }
}
impl AbiDecode for str[49] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[49] {
        let data = buffer.read_bytes(49);
        asm(s: data.ptr()) {
            s: str[49]
        }
    }
}
impl AbiDecode for str[50] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[50] {
        let data = buffer.read_bytes(50);
        asm(s: data.ptr()) {
            s: str[50]
        }
    }
}
impl AbiDecode for str[51] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[51] {
        let data = buffer.read_bytes(51);
        asm(s: data.ptr()) {
            s: str[51]
        }
    }
}
impl AbiDecode for str[52] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[52] {
        let data = buffer.read_bytes(52);
        asm(s: data.ptr()) {
            s: str[52]
        }
    }
}
impl AbiDecode for str[53] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[53] {
        let data = buffer.read_bytes(53);
        asm(s: data.ptr()) {
            s: str[53]
        }
    }
}
impl AbiDecode for str[54] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[54] {
        let data = buffer.read_bytes(54);
        asm(s: data.ptr()) {
            s: str[54]
        }
    }
}
impl AbiDecode for str[55] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[55] {
        let data = buffer.read_bytes(55);
        asm(s: data.ptr()) {
            s: str[55]
        }
    }
}
impl AbiDecode for str[56] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[56] {
        let data = buffer.read_bytes(56);
        asm(s: data.ptr()) {
            s: str[56]
        }
    }
}
impl AbiDecode for str[57] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[57] {
        let data = buffer.read_bytes(57);
        asm(s: data.ptr()) {
            s: str[57]
        }
    }
}
impl AbiDecode for str[58] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[58] {
        let data = buffer.read_bytes(58);
        asm(s: data.ptr()) {
            s: str[58]
        }
    }
}
impl AbiDecode for str[59] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[59] {
        let data = buffer.read_bytes(59);
        asm(s: data.ptr()) {
            s: str[59]
        }
    }
}
impl AbiDecode for str[60] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[60] {
        let data = buffer.read_bytes(60);
        asm(s: data.ptr()) {
            s: str[60]
        }
    }
}
impl AbiDecode for str[61] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[61] {
        let data = buffer.read_bytes(61);
        asm(s: data.ptr()) {
            s: str[61]
        }
    }
}
impl AbiDecode for str[62] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[62] {
        let data = buffer.read_bytes(62);
        asm(s: data.ptr()) {
            s: str[62]
        }
    }
}
impl AbiDecode for str[63] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[63] {
        let data = buffer.read_bytes(63);
        asm(s: data.ptr()) {
            s: str[63]
        }
    }
}
impl AbiDecode for str[64] {
    fn abi_decode(ref mut buffer: BufferReader) -> str[64] {
        let data = buffer.read_bytes(64);
        asm(s: data.ptr()) {
            s: str[64]
        }
    }
}
// END STRARRAY_DECODE

#[cfg(experimental_const_generics = true)]
impl<T, const N: u64> AbiDecode for [T; N]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; N] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; N];
        let mut i = 1;
        while i < N {
            let item: &mut T = __elem_at(&mut array, i);
            *item = buffer.decode::<T>();
            i += 1;
        }
        array
    }
}

// BEGIN ARRAY_DECODE
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 1]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 1] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 1];
        let mut i = 1;
        while i < 1 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 2]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 2] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 2];
        let mut i = 1;
        while i < 2 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 3]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 3] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 3];
        let mut i = 1;
        while i < 3 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 4]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 4] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 4];
        let mut i = 1;
        while i < 4 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 5]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 5] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 5];
        let mut i = 1;
        while i < 5 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 6]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 6] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 6];
        let mut i = 1;
        while i < 6 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 7]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 7] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 7];
        let mut i = 1;
        while i < 7 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 8]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 8] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 8];
        let mut i = 1;
        while i < 8 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 9]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 9] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 9];
        let mut i = 1;
        while i < 9 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 10]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 10] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 10];
        let mut i = 1;
        while i < 10 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 11]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 11] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 11];
        let mut i = 1;
        while i < 11 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 12]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 12] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 12];
        let mut i = 1;
        while i < 12 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 13]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 13] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 13];
        let mut i = 1;
        while i < 13 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 14]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 14] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 14];
        let mut i = 1;
        while i < 14 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 15]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 15] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 15];
        let mut i = 1;
        while i < 15 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 16]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 16] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 16];
        let mut i = 1;
        while i < 16 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 17]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 17] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 17];
        let mut i = 1;
        while i < 17 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 18]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 18] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 18];
        let mut i = 1;
        while i < 18 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 19]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 19] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 19];
        let mut i = 1;
        while i < 19 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 20]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 20] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 20];
        let mut i = 1;
        while i < 20 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 21]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 21] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 21];
        let mut i = 1;
        while i < 21 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 22]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 22] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 22];
        let mut i = 1;
        while i < 22 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 23]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 23] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 23];
        let mut i = 1;
        while i < 23 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 24]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 24] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 24];
        let mut i = 1;
        while i < 24 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 25]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 25] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 25];
        let mut i = 1;
        while i < 25 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 26]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 26] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 26];
        let mut i = 1;
        while i < 26 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 27]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 27] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 27];
        let mut i = 1;
        while i < 27 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 28]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 28] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 28];
        let mut i = 1;
        while i < 28 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 29]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 29] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 29];
        let mut i = 1;
        while i < 29 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 30]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 30] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 30];
        let mut i = 1;
        while i < 30 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 31]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 31] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 31];
        let mut i = 1;
        while i < 31 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 32]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 32] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 32];
        let mut i = 1;
        while i < 32 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 33]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 33] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 33];
        let mut i = 1;
        while i < 33 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 34]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 34] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 34];
        let mut i = 1;
        while i < 34 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 35]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 35] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 35];
        let mut i = 1;
        while i < 35 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 36]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 36] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 36];
        let mut i = 1;
        while i < 36 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 37]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 37] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 37];
        let mut i = 1;
        while i < 37 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 38]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 38] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 38];
        let mut i = 1;
        while i < 38 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 39]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 39] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 39];
        let mut i = 1;
        while i < 39 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 40]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 40] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 40];
        let mut i = 1;
        while i < 40 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 41]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 41] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 41];
        let mut i = 1;
        while i < 41 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 42]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 42] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 42];
        let mut i = 1;
        while i < 42 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 43]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 43] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 43];
        let mut i = 1;
        while i < 43 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 44]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 44] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 44];
        let mut i = 1;
        while i < 44 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 45]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 45] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 45];
        let mut i = 1;
        while i < 45 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 46]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 46] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 46];
        let mut i = 1;
        while i < 46 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 47]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 47] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 47];
        let mut i = 1;
        while i < 47 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 48]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 48] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 48];
        let mut i = 1;
        while i < 48 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 49]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 49] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 49];
        let mut i = 1;
        while i < 49 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 50]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 50] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 50];
        let mut i = 1;
        while i < 50 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 51]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 51] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 51];
        let mut i = 1;
        while i < 51 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 52]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 52] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 52];
        let mut i = 1;
        while i < 52 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 53]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 53] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 53];
        let mut i = 1;
        while i < 53 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 54]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 54] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 54];
        let mut i = 1;
        while i < 54 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 55]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 55] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 55];
        let mut i = 1;
        while i < 55 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 56]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 56] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 56];
        let mut i = 1;
        while i < 56 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 57]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 57] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 57];
        let mut i = 1;
        while i < 57 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 58]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 58] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 58];
        let mut i = 1;
        while i < 58 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 59]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 59] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 59];
        let mut i = 1;
        while i < 59 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 60]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 60] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 60];
        let mut i = 1;
        while i < 60 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 61]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 61] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 61];
        let mut i = 1;
        while i < 61 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 62]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 62] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 62];
        let mut i = 1;
        while i < 62 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 63]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 63] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 63];
        let mut i = 1;
        while i < 63 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
#[cfg(experimental_const_generics = false)]
impl<T> AbiDecode for [T; 64]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 64] {
        let first: T = buffer.decode::<T>();
        let mut array = [first; 64];
        let mut i = 1;
        while i < 64 {
            array[i] = buffer.decode::<T>();
            i += 1;
        };
        array
    }
}
// END ARRAY_DECODE

impl AbiDecode for () {
    fn abi_decode(ref mut _buffer: BufferReader) -> () {
        ()
    }
}

// BEGIN TUPLES_DECODE
impl<A> AbiDecode for (A, )
where
    A: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> Self {
        (A::abi_decode(buffer), )
    }
}
impl<A, B> AbiDecode for (A, B)
where
    A: AbiDecode,
    B: AbiDecode,
{
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
    method_name: str,
    args: TArgs,
    coins: u64,
    asset_id: b256,
    gas: u64,
) -> T
where
    T: AbiDecode,
    TArgs: AbiEncode,
{
    let first_parameter = encode(method_name);
    let second_parameter = encode(args);
    let params = encode((
        contract_id,
        asm(a: first_parameter.ptr()) {
            a: u64
        },
        asm(a: second_parameter.ptr()) {
            a: u64
        },
    ));

    __contract_call(params.ptr(), coins, asset_id, gas);
    let ptr = asm() {
        ret: raw_ptr
    };
    let len = asm() {
        retl: u64
    };

    let mut buffer = BufferReader::from_parts(ptr, len);
    T::abi_decode(buffer)
}

pub fn decode_script_data<T>() -> T
where
    T: AbiDecode,
{
    let mut buffer = BufferReader::from_script_data();
    T::abi_decode(buffer)
}

pub fn decode_predicate_data<T>() -> T
where
    T: AbiDecode,
{
    let mut buffer = BufferReader::from_predicate_data();
    T::abi_decode(buffer)
}

pub fn decode_predicate_data_by_index<T>(index: u64) -> T
where
    T: AbiDecode,
{
    let mut buffer = BufferReader::from_predicate_data_by_index(index);
    T::abi_decode(buffer)
}

pub fn decode_first_param<T>() -> T
where
    T: AbiDecode,
{
    let mut buffer = BufferReader::from_first_parameter();
    T::abi_decode(buffer)
}

pub fn decode_second_param<T>() -> T
where
    T: AbiDecode,
{
    let mut buffer = BufferReader::from_second_parameter();
    T::abi_decode(buffer)
}

// Tests


fn assert_encoding<T, SLICE>(value: T, expected: SLICE)
where
    T: AbiEncode,
{
    let len = __size_of::<SLICE>();

    if len == 0 {
        __revert(0);
    }

    let expected = raw_slice::from_parts::<u8>(__addr_of(expected), len);
    let actual = encode(value);

    if actual.len::<u8>() != expected.len::<u8>() {
        __revert(0);
    }

    let result = asm(
        result,
        expected: expected.ptr(),
        actual: actual.ptr(),
        len: len,
    ) {
        meq result expected actual len;
        result: bool
    };

    if !result {
        __revert(0);
    }
}

fn assert_encoding_and_decoding<T, SLICE>(
    value: T,
    expected: SLICE,
)
where
    T: PartialEq + AbiEncode + AbiDecode,
{
    let len = __size_of::<SLICE>();

    if len == 0 {
        __revert(0);
    }

    let expected = raw_slice::from_parts::<u8>(__addr_of(expected), len);
    let actual = encode(value);

    if actual.len::<u8>() != expected.len::<u8>() {
        __revert(0);
    }

    let result = asm(
        result,
        expected: expected.ptr(),
        actual: actual.ptr(),
        len: len,
    ) {
        meq result expected actual len;
        result: bool
    };

    if !result {
        __revert(0);
    }

    let decoded = abi_decode::<T>(actual);
    __log(decoded);
    if !decoded.eq(value) {
        __revert(0);
    }
}

fn to_slice<T>(array: T) -> raw_slice {
    let len = __size_of::<T>();
    raw_slice::from_parts::<u8>(__addr_of(array), len)
}

fn assert_ge<T>(a: T, b: T, revert_code: u64)
where
    T: Ord,
{
    if a.lt(b) {
        __revert(revert_code)
    }
}

fn assert_eq<T>(a: T, b: T, revert_code: u64)
where
    T: PartialEq,
{
    if a != b {
        __revert(revert_code)
    }
}

fn assert_neq<T>(a: T, b: T, revert_code: u64)
where
    T: PartialEq,
{
    if a == b {
        __revert(revert_code)
    }
}

fn assert_no_write_after_buffer<T>(value_to_append: T, size_of_t: u64)
where
    T: AbiEncode,
{
    // This red zone should not be overwritten
    let red_zone1 = asm(size: 1024) {
        aloc size;
        hp: raw_ptr
    };
    red_zone1.write(0xFFFFFFFFFFFFFFFF);

    // Create encoding buffer with capacity for one item
    let mut buffer = Buffer::with_capacity(size_of_t);
    let ptr1 = buffer.buffer.0;

    // Append one item
    let buffer = value_to_append.abi_encode(buffer);
    assert_eq(ptr1, buffer.buffer.0, 1); // no buffer grow is expected
    assert_eq(buffer.buffer.1, size_of_t, 2); // capacity must be still be one item
    assert_eq(buffer.buffer.2, size_of_t, 3); // buffer has one item

    // This red zone should not be overwritten
    let red_zone2 = asm(size: 1024) {
        aloc size;
        hp: raw_ptr
    };
    red_zone2.write(0xFFFFFFFFFFFFFFFF);

    // Append another item
    let buffer = value_to_append.abi_encode(buffer);
    assert_neq(ptr1, buffer.buffer.0, 4); // must have allocated new buffer
    assert_ge(buffer.buffer.1, size_of_t * 2, 5); // capacity for at least two items
    assert_eq(buffer.buffer.2, size_of_t * 2, 6); // buffer has two items

    // Check that red zones were not overwritten
    assert_eq(red_zone1.read::<u64>(), 0xFFFFFFFFFFFFFFFF, 7);
    assert_eq(red_zone2.read::<u64>(), 0xFFFFFFFFFFFFFFFF, 8);
}

#[test]
fn ok_encoding_should_not_write_outside_buffer() {
    assert_no_write_after_buffer::<bool>(true, 1);

    // numbers
    assert_no_write_after_buffer::<u8>(1, 1);
    assert_no_write_after_buffer::<u16>(1, 2);
    assert_no_write_after_buffer::<u32>(1, 4);
    assert_no_write_after_buffer::<u64>(1, 8);
    assert_no_write_after_buffer::<u256>(
        0x0000000000000000000000000000000000000000000000000000000000000001u256,
        32,
    );
    assert_no_write_after_buffer::<b256>(
        0x0000000000000000000000000000000000000000000000000000000000000001,
        32,
    );

    // arrays
    assert_no_write_after_buffer::<[u8; 1]>([1], 1);
    assert_no_write_after_buffer::<[u8; 2]>([1, 1], 2);
    assert_no_write_after_buffer::<[u8; 3]>([1, 1, 1], 3);
    assert_no_write_after_buffer::<[u8; 4]>([1, 1, 1, 1], 4);
    assert_no_write_after_buffer::<[u8; 5]>([1, 1, 1, 1, 1], 5);

    // string arrays
    assert_no_write_after_buffer::<str[1]>(__to_str_array("h"), 1);
    assert_no_write_after_buffer::<str[2]>(__to_str_array("he"), 2);
    assert_no_write_after_buffer::<str[11]>(__to_str_array("hello world"), 11);

    // string slices
    assert_no_write_after_buffer::<str>("h", 9);
    assert_no_write_after_buffer::<str>("he", 10);
    assert_no_write_after_buffer::<str>("hello world", 19);
}

#[test]
fn ok_abi_encoding() {
    // bool
    assert_encoding_and_decoding(false, [0u8]);
    assert_encoding_and_decoding(true, [1u8]);

    // numbers
    assert_encoding_and_decoding(0u8, [0u8]);
    assert_encoding_and_decoding(255u8, [255u8]);

    assert_encoding_and_decoding(0u16, [0u8, 0u8]);
    assert_encoding_and_decoding(128u16, [0u8, 128u8]);
    assert_encoding_and_decoding(65535u16, [255u8, 255u8]);

    assert_encoding_and_decoding(0u32, [0u8, 0u8, 0u8, 0u8]);
    assert_encoding_and_decoding(128u32, [0u8, 0u8, 0u8, 128u8]);
    assert_encoding_and_decoding(4294967295u32, [255u8, 255u8, 255u8, 255u8]);

    assert_encoding_and_decoding(0u64, [0u8; 8]);
    assert_encoding_and_decoding(128u64, [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 128u8]);
    assert_encoding_and_decoding(18446744073709551615u64, [255u8; 8]);

    assert_encoding_and_decoding(
        0x0000000000000000000000000000000000000000000000000000000000000000u256,
        [0u8; 32],
    );
    assert_encoding_and_decoding(
        0xAA000000000000000000000000000000000000000000000000000000000000BBu256,
        [
            0xAAu8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0xBBu8,
        ],
    );
    assert_encoding_and_decoding(
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu256,
        [255u8; 32],
    );

    assert_encoding_and_decoding(
        0x0000000000000000000000000000000000000000000000000000000000000000,
        [0u8; 32],
    );
    assert_encoding_and_decoding(
        0xAA000000000000000000000000000000000000000000000000000000000000BB,
        [
            0xAAu8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0xBBu8,
        ],
    );
    assert_encoding_and_decoding(
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        [255u8; 32],
    );

    // strings
    assert_encoding_and_decoding(
        "Hello",
        [0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 5u8, 72u8, 101u8, 108u8, 108u8, 111u8],
    );

    assert_encoding(
        {
            let a: str[1] = __to_str_array("a");
            a
        },
        [97u8],
    );
    assert_encoding(
        {
            let a: str[2] = __to_str_array("aa");
            a
        },
        [97u8, 97u8],
    );
    assert_encoding(
        {
            let a: str[3] = __to_str_array("aaa");
            a
        },
        [97u8, 97u8, 97u8],
    );
    assert_encoding(
        {
            let a: str[4] = __to_str_array("aaaa");
            a
        },
        [97u8, 97u8, 97u8, 97u8],
    );
    assert_encoding(
        {
            let a: str[5] = __to_str_array("aaaaa");
            a
        },
        [97u8, 97u8, 97u8, 97u8, 97u8],
    );

    // arrays
    assert_encoding([255u8; 1], [255u8; 1]);
    assert_encoding([255u8; 2], [255u8; 2]);
    assert_encoding([255u8; 3], [255u8; 3]);
    assert_encoding([255u8; 4], [255u8; 4]);
    assert_encoding([255u8; 5], [255u8; 5]);

    let array = abi_decode::<[u8; 1]>(to_slice([255u8]));
    assert_eq(array[0], 255u8, 0);

    let array = abi_decode::<[u8; 2]>(to_slice([255u8, 254u8]));
    assert_eq(array[0], 255u8, 0);
    assert_eq(array[1], 254u8, 0);
}

#[test(should_revert)]
fn nok_abi_encoding_invalid_bool() {
    let actual = encode(2u8);
    let _ = abi_decode::<bool>(actual);
}
