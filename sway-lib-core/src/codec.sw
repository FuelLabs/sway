library;

use ::raw_slice::*;

pub struct Buffer {
    buffer: raw_ptr,
    cap: u64,
    size: u64,
}

impl Buffer {
    pub fn new() -> Self {
        let cap = 1024;
        Buffer {
            buffer: asm(size: cap) {
                aloc size;
                hp: raw_ptr
            },
            cap,
            size: 0,
        }
    }

    pub fn push<T>(ref mut self, val: T) {
        let count = __size_of::<T>();

        if self.cap >= self.size + count {
            self.buffer.add::<u8>(self.size).write(val);
            self.size += count;
        } else {
            __revert(123456789);
        }
    }
}

impl AsRawSlice for Buffer {
    fn as_raw_slice(self) -> raw_slice {
        asm(ptr: (self.buffer, self.size)) {
            ptr: raw_slice
        }
    }
}

pub struct BufferReader {
    ptr: raw_ptr,
    pos: u64
}

impl BufferReader {
    pub fn from_parts(ptr: raw_ptr, _len: u64) -> BufferReader {
        BufferReader {
            ptr,
            pos: 0,
        }
    }

    pub fn from_first_parameter() -> BufferReader {
        const FIRST_PARAMETER_OFFSET: u64 = 73;

        let ptr = asm() {
            fp: raw_ptr
        };
        let ptr = ptr.add::<u64>(FIRST_PARAMETER_OFFSET);

        BufferReader {
            ptr,
            pos: 0,
        }
    }

    pub fn from_second_parameter() -> BufferReader {
        const SECOND_PARAMETER_OFFSET: u64 = 74;

        let ptr = asm() {
            fp: raw_ptr
        };
        let ptr = ptr.add::<u64>(SECOND_PARAMETER_OFFSET);

        BufferReader {
            ptr,
            pos: 0,
        }
    }

    pub fn from_script_data() -> BufferReader {
        let ptr = __gtf::<raw_ptr>(0, 0xA); // SCRIPT_DATA
        let _len = __gtf::<u64>(0, 0x4); // SCRIPT_DATA_LEN
        BufferReader {
            ptr,
            pos: 0,
        }
    }

    pub fn read_bytes(ref mut self, count: u64) -> raw_slice {
        let next_pos = self.pos + count;

        let ptr = self.ptr.add::<u8>(self.pos);
        let slice =  asm(ptr: (ptr, count)) {
            ptr: raw_slice
        };

        self.pos = next_pos;

        slice
    }

    pub fn read<T>(ref mut self) -> T {
        let ptr = self.ptr.add::<u8>(self.pos);

        let size = __size_of::<T>();
        let next_pos = self.pos + size;

        if __is_reference_type::<T>() {
            let v = asm(ptr: ptr) {
                ptr: T
            };
            self.pos = next_pos;
            v
        } else if size == 1 {
            let v = asm(ptr: ptr, val) {
                lb val ptr i0;
                val: T
            };
            self.pos = next_pos;
            v
        } else {
            let v = asm(ptr: ptr, val) {
                lw val ptr i0;
                val: T
            };
            self.pos = next_pos;
            v
        }
    }

    pub fn decode<T>(ref mut self) -> T 
    where
        T: AbiDecode
    {
        T::abi_decode(self)
    }
}

// Encode

pub trait AbiEncode {
    fn abi_encode(self, ref mut buffer: Buffer);
}

impl AbiEncode for bool {
    fn abi_encode(self, ref mut buffer: Buffer) {
        buffer.push(self);
    }
}

// Encode Numbers

impl AbiEncode for b256 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) {
            r1: (u64, u64, u64, u64)
        };
        buffer.push(a);
        buffer.push(b);
        buffer.push(c);
        buffer.push(d);
    }
}

impl AbiEncode for u256 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) {
            r1: (u64, u64, u64, u64)
        };
        buffer.push(a);
        buffer.push(b);
        buffer.push(c);
        buffer.push(d);
    }
}

impl AbiEncode for u64 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        buffer.push(self);
    }
}

impl AbiEncode for u32 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        let output = [0_u8, 0_u8, 0_u8, 0_u8];
        let output = asm(
            input: self,
            off: 0xFF,
            i: 0x8,
            j: 0x10,
            k: 0x18,
            output: output,
            r1,
        ) {
            and r1 input off;
            sb output r1 i0;

            srl r1 input i;
            and r1 r1 off;
            sb output r1 i1;

            srl r1 input j;
            and r1 r1 off;
            sb output r1 i2;

            srl r1 input k;
            and r1 r1 off;
            sb output r1 i3;

            output: [u8; 4]
        };

        buffer.push(output[3]);
        buffer.push(output[2]);
        buffer.push(output[1]);
        buffer.push(output[0]);
    }
}

impl AbiEncode for u16 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        let output = [0_u8, 0_u8];
        let output = asm(input: self, off: 0xFF, i: 0x8, output: output, r1) {
            and r1 input off;
            sb output r1 i0;

            srl r1 input i;
            and r1 r1 off;
            sb output r1 i1;

            output: [u8; 2]
        };

        buffer.push(output[1]);
        buffer.push(output[0]);
    }
}

impl AbiEncode for u8 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        buffer.push(self);
    }
}

// Encode str slice and str arrays

impl AbiEncode for str {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let len = self.len();
        buffer.push(len);

        let ptr = self.as_ptr();

        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push(byte);
            i += 1;
        }
    }
}

impl AbiEncode for str[0] { fn abi_encode(self, ref mut _buffer: Buffer) {} }

// BEGIN STRARRAY_ENCODE
impl AbiEncode for str[1] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[2] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[3] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[4] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[5] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[6] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[7] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[8] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[9] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[10] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[11] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[12] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[13] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[14] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[15] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
impl AbiEncode for str[16] { fn abi_encode(self, ref mut buffer: Buffer) { use ::str::*; let s = from_str_array(self); let len = s.len(); let ptr = s.as_ptr(); let mut i = 0; while i < len { let byte = ptr.add::<u8>(i).read::<u8>(); buffer.push(byte); i += 1; } } }
// END STRARRAY_ENCODE

// Encode Arrays and Slices

impl AbiEncode for raw_slice {
    fn abi_encode(self, ref mut buffer: Buffer) {
        let len = self.number_of_bytes();
        buffer.push(len);

        let ptr = self.ptr();

        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push(byte);
            i += 1;
        }
    }
}

impl<T> AbiEncode for [T; 0]
where
    T: AbiEncode,
{
    fn abi_encode(self, ref mut _buffer: Buffer) {}
}

impl<T> AbiEncode for [T; 1]
where
    T: AbiEncode,
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self[0].abi_encode(buffer);
    }
}

impl<T> AbiEncode for [T; 2]
where
    T: AbiEncode,
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self[0].abi_encode(buffer);
        self[1].abi_encode(buffer);
    }
}

impl<T> AbiEncode for [T; 3]
where
    T: AbiEncode,
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self[0].abi_encode(buffer);
        self[1].abi_encode(buffer);
        self[2].abi_encode(buffer);
    }
}

impl<T> AbiEncode for [T; 4]
where
    T: AbiEncode,
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self[0].abi_encode(buffer);
        self[1].abi_encode(buffer);
        self[2].abi_encode(buffer);
        self[3].abi_encode(buffer);
    }
}

impl<T> AbiEncode for [T; 5]
where
    T: AbiEncode,
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self[0].abi_encode(buffer);
        self[1].abi_encode(buffer);
        self[2].abi_encode(buffer);
        self[3].abi_encode(buffer);
        self[4].abi_encode(buffer);
    }
}

// Encode Tuples

impl AbiEncode for ()
{
    fn abi_encode(self, ref mut _buffer: Buffer) {
    }
}

impl<A> AbiEncode for (A,)
where
    A: AbiEncode
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
    }
}

impl<A, B> AbiEncode for (A, B)
where
    A: AbiEncode,
    B: AbiEncode,
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
    }
}

impl<A, B, C> AbiEncode for (A, B, C)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
    }
}

impl<A, B, C, D> AbiEncode for (A, B, C, D)
where
    A: AbiEncode,
    B: AbiEncode,
    C: AbiEncode,
    D: AbiEncode,
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
    }
}

pub fn encode<T>(item: T) -> raw_slice
where
    T: AbiEncode
{
    let mut buffer = Buffer::new();
    item.abi_encode(buffer);
    buffer.as_raw_slice()
}

pub fn abi_decode<T>(data: raw_slice) -> T
where
    T: AbiDecode
{
    let mut buffer = BufferReader::from_parts(data.ptr(), data.len::<u8>());
    T::abi_decode(buffer)
}

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

// Decode 

pub trait AbiDecode {
    fn abi_decode(ref mut buffer: BufferReader) -> Self;
}

impl AbiDecode for b256 {
    fn abi_decode(ref mut buffer: BufferReader) -> b256 {
        buffer.read::<b256>()
    }
}

impl AbiDecode for u256 {
    fn abi_decode(ref mut buffer: BufferReader) -> u256 {
        buffer.read::<u256>()
    }
}

impl AbiDecode for u64 {
    fn abi_decode(ref mut buffer: BufferReader) -> u64 {
        buffer.read::<u64>()
    }
}

impl AbiDecode for u32 {
    fn abi_decode(ref mut buffer: BufferReader) -> u32 {
        buffer.read::<u32>()
    }
}

impl AbiDecode for u16 {
    fn abi_decode(ref mut buffer: BufferReader) -> u16 {
        buffer.read::<u16>()
    }
}

impl AbiDecode for u8 {
    fn abi_decode(ref mut buffer: BufferReader) -> u8 {
        buffer.read::<u8>()
    }
}

impl AbiDecode for bool {
    fn abi_decode(ref mut buffer: BufferReader) -> bool {
        buffer.read::<bool>()
    }
}

impl AbiDecode for raw_slice {
    fn abi_decode(ref mut buffer: BufferReader) -> raw_slice {
        let len = u64::abi_decode(buffer);
        let data = buffer.read_bytes(len);
        asm(s: (data.ptr(), len)) {
            s: raw_slice
        }
    }
}

impl AbiDecode for str {
    fn abi_decode(ref mut buffer: BufferReader) -> str {
        let len = u64::abi_decode(buffer);
        let data = buffer.read_bytes(len);
        asm(s: (data.ptr(), len)) {
            s: str
        }
    }
}

// BEGIN STRARRAY_DECODE
impl AbiDecode for str[1] { fn abi_decode(ref mut buffer: BufferReader) -> str[1] { let data = buffer.read_bytes(1); asm(s: data.ptr()) { s: str[1] } } }
impl AbiDecode for str[2] { fn abi_decode(ref mut buffer: BufferReader) -> str[2] { let data = buffer.read_bytes(2); asm(s: data.ptr()) { s: str[2] } } }
impl AbiDecode for str[3] { fn abi_decode(ref mut buffer: BufferReader) -> str[3] { let data = buffer.read_bytes(3); asm(s: data.ptr()) { s: str[3] } } }
impl AbiDecode for str[4] { fn abi_decode(ref mut buffer: BufferReader) -> str[4] { let data = buffer.read_bytes(4); asm(s: data.ptr()) { s: str[4] } } }
impl AbiDecode for str[5] { fn abi_decode(ref mut buffer: BufferReader) -> str[5] { let data = buffer.read_bytes(5); asm(s: data.ptr()) { s: str[5] } } }
impl AbiDecode for str[6] { fn abi_decode(ref mut buffer: BufferReader) -> str[6] { let data = buffer.read_bytes(6); asm(s: data.ptr()) { s: str[6] } } }
impl AbiDecode for str[7] { fn abi_decode(ref mut buffer: BufferReader) -> str[7] { let data = buffer.read_bytes(7); asm(s: data.ptr()) { s: str[7] } } }
impl AbiDecode for str[8] { fn abi_decode(ref mut buffer: BufferReader) -> str[8] { let data = buffer.read_bytes(8); asm(s: data.ptr()) { s: str[8] } } }
impl AbiDecode for str[9] { fn abi_decode(ref mut buffer: BufferReader) -> str[9] { let data = buffer.read_bytes(9); asm(s: data.ptr()) { s: str[9] } } }
impl AbiDecode for str[10] { fn abi_decode(ref mut buffer: BufferReader) -> str[10] { let data = buffer.read_bytes(10); asm(s: data.ptr()) { s: str[10] } } }
impl AbiDecode for str[11] { fn abi_decode(ref mut buffer: BufferReader) -> str[11] { let data = buffer.read_bytes(11); asm(s: data.ptr()) { s: str[11] } } }
impl AbiDecode for str[12] { fn abi_decode(ref mut buffer: BufferReader) -> str[12] { let data = buffer.read_bytes(12); asm(s: data.ptr()) { s: str[12] } } }
impl AbiDecode for str[13] { fn abi_decode(ref mut buffer: BufferReader) -> str[13] { let data = buffer.read_bytes(13); asm(s: data.ptr()) { s: str[13] } } }
impl AbiDecode for str[14] { fn abi_decode(ref mut buffer: BufferReader) -> str[14] { let data = buffer.read_bytes(14); asm(s: data.ptr()) { s: str[14] } } }
impl AbiDecode for str[15] { fn abi_decode(ref mut buffer: BufferReader) -> str[15] { let data = buffer.read_bytes(15); asm(s: data.ptr()) { s: str[15] } } }
impl AbiDecode for str[16] { fn abi_decode(ref mut buffer: BufferReader) -> str[16] { let data = buffer.read_bytes(16); asm(s: data.ptr()) { s: str[16] } } }
// END STRARRAY_DECODE

impl<T> AbiDecode for [T; 0]
where
    T: AbiDecode
{
    fn abi_decode(ref mut _buffer: BufferReader) -> [T; 0] {
        []
    }
}

impl<T> AbiDecode for [T; 1]
where
    T: AbiDecode
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 1] {
        [
            T::abi_decode(buffer)
        ]
    }
}

impl<T> AbiDecode for [T; 2]
where
    T: AbiDecode
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 2] {
        [
            T::abi_decode(buffer),
            T::abi_decode(buffer)
        ]
    }
}

impl<T> AbiDecode for [T; 3]
where
    T: AbiDecode
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 3] {
        [
            T::abi_decode(buffer),
            T::abi_decode(buffer),
            T::abi_decode(buffer)
        ]
    }
}

impl<T> AbiDecode for [T; 4]
where
    T: AbiDecode
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 4] {
        [
            T::abi_decode(buffer),
            T::abi_decode(buffer),
            T::abi_decode(buffer),
            T::abi_decode(buffer),
        ]
    }
}

impl AbiDecode for () {
    fn abi_decode(ref mut _buffer: BufferReader) -> () {
        ()
    }
}

impl<A> AbiDecode for (A,) 
where
    A: AbiDecode 
{
    fn abi_decode(ref mut buffer: BufferReader) -> (A,) {
        let a = A::abi_decode(buffer);
        (a,)
    }
}

impl<A, B> AbiDecode for (A, B) 
where
    A: AbiDecode,
    B: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> (A, B) {
        let a = A::abi_decode(buffer);
        let b = B::abi_decode(buffer);
        (a, b)
    }
}

impl<A, B, C> AbiDecode for (A, B, C) 
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> (A, B, C) {
        let a = A::abi_decode(buffer);
        let b = B::abi_decode(buffer);
        let c = C::abi_decode(buffer);
        (a, b, c)
    }
}

impl<A, B, C, D> AbiDecode for (A, B, C, D) 
where
    A: AbiDecode,
    B: AbiDecode,
    C: AbiDecode,
    D: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> (A, B, C, D) {
        let a = A::abi_decode(buffer);
        let b = B::abi_decode(buffer);
        let c = C::abi_decode(buffer);
        let d = D::abi_decode(buffer);
        (a, b, c, d)
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
    fn abi_decode(ref mut buffer: BufferReader) -> (A, B, C, D, E, F, G, H, I, J, K) {
        let a = A::abi_decode(buffer);
        let b = B::abi_decode(buffer);
        let c = C::abi_decode(buffer);
        let d = D::abi_decode(buffer);
        let e = E::abi_decode(buffer);
        let f = F::abi_decode(buffer);
        let g = G::abi_decode(buffer);
        let h = H::abi_decode(buffer);
        let i = I::abi_decode(buffer);
        let j = J::abi_decode(buffer);
        let k = K::abi_decode(buffer);
        (a, b, c, d, e, f, g, h, i, j, k)
    }
}


#[test]
fn ok_encode() {
    // bool
    assert_encoding(false, [0u8]);
    assert_encoding(true, [1u8]);

    // numbers
    assert_encoding(0u8, [0u8; 1]);
    assert_encoding(255u8, [255u8; 1]);
    assert_encoding(0u16, [0u8; 2]);
    assert_encoding(65535u16, [255u8; 2]);
    assert_encoding(0u32, [0u8; 4]);
    assert_encoding(4294967295u32, [255u8; 4]);
    assert_encoding(0u64, [0u8; 8]);
    assert_encoding(18446744073709551615u64, [255u8; 8]);
    assert_encoding(
        0x0000000000000000000000000000000000000000000000000000000000000000u256,
        [0u8; 32],
    );
    assert_encoding(
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu256,
        [255u8; 32],
    );
    assert_encoding(
        0x0000000000000000000000000000000000000000000000000000000000000000,
        [0u8; 32],
    );
    assert_encoding(
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
        [255u8; 32],
    );

    // strings
    assert_encoding(
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
}

pub fn contract_call<T, TArgs>(contract_id: b256, method_name: str, args: TArgs, coins: u64, asset_id: b256, gas: u64) -> T
where
    T: AbiDecode,
    TArgs: AbiEncode
{
    let first_parameter = encode(method_name);
    let second_parameter = encode(args);
    let params = encode(
        (
            contract_id,
            asm(a: first_parameter.ptr()) { a: u64 },
            asm(a: second_parameter.ptr()) { a: u64 },
        )
    );

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
    T: AbiDecode
{
    let mut buffer = BufferReader::from_script_data();
    T::abi_decode(buffer)
}

pub fn decode_first_param<T>() -> T 
where
    T: AbiDecode
{
    let mut buffer = BufferReader::from_first_parameter();
    T::abi_decode(buffer)
}

pub fn decode_second_param<T>() -> T 
where
    T: AbiDecode
{
    let mut buffer = BufferReader::from_second_parameter();
    T::abi_decode(buffer)
}