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

    pub fn push_byte(ref mut self, val: u8) {
        let count = 1;
        if self.cap >= self.size + count {
            let ptr = self.buffer.add::<u8>(self.size);
            asm(ptr: ptr, val: val) {
                sb ptr val i0;
            };
            self.size += count;
        } else {
            __revert(123456789);
        }
    }

    pub fn push_u64(ref mut self, val: u64) {
        let count = 8;
        if self.cap >= self.size + count {
            let ptr = self.buffer.add::<u8>(self.size);
            asm(ptr: ptr, val: val) {
                sw ptr val i0;
            };
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
    pos: u64,
}

impl BufferReader {
    pub fn from_parts(ptr: raw_ptr, _len: u64) -> BufferReader {
        BufferReader { ptr, pos: 0 }
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
            pos: 0,
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
            pos: 0,
        }
    }

    pub fn from_script_data() -> BufferReader {
        let ptr = __gtf::<raw_ptr>(0, 0xA); // SCRIPT_DATA
        let _len = __gtf::<u64>(0, 0x4); // SCRIPT_DATA_LEN
        BufferReader { ptr, pos: 0 }
    }

    pub fn from_predicate_data() -> BufferReader {
        let predicate_index = asm(r1) {
            gm r1 i3; // GET_VERIFYING_PREDICATE
            r1: u64
        };
        match __gtf::<u8>(predicate_index, 0x200) { // GTF_INPUT_TYPE
            0u8 => {
                let ptr = __gtf::<raw_ptr>(predicate_index, 0x20C); // INPUT_COIN_PREDICATE_DATA
                let _len = __gtf::<u64>(predicate_index, 0x20A); // INPUT_COIN_PREDICATE_DATA_LENGTH
                BufferReader { ptr, pos: 0 }
            },
            2u8 => {
                let ptr = __gtf::<raw_ptr>(predicate_index, 0x24A); // INPUT_MESSAGE_PREDICATE_DATA
                let _len = __gtf::<u64>(predicate_index, 0x247); // INPUT_MESSAGE_PREDICATE_DATA_LENGTH
                BufferReader { ptr, pos: 0 }
            },
            _ => __revert(0),
        }
    }

    pub fn read_bytes(ref mut self, count: u64) -> raw_slice {
        let next_pos = self.pos + count;

        let ptr = self.ptr.add::<u8>(self.pos);
        let slice = asm(ptr: (ptr, count)) {
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
        T: AbiDecode,
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
        buffer.push_byte(if self { 1 } else { 0 });
    }
}

// Encode Numbers

impl AbiEncode for b256 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) {
            r1: (u64, u64, u64, u64)
        };
        buffer.push_u64(a);
        buffer.push_u64(b);
        buffer.push_u64(c);
        buffer.push_u64(d);
    }
}

impl AbiEncode for u256 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        let (a, b, c, d): (u64, u64, u64, u64) = asm(r1: self) {
            r1: (u64, u64, u64, u64)
        };
        buffer.push_u64(a);
        buffer.push_u64(b);
        buffer.push_u64(c);
        buffer.push_u64(d);
    }
}

impl AbiEncode for u64 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        buffer.push_u64(self);
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

        buffer.push_byte(output[3]);
        buffer.push_byte(output[2]);
        buffer.push_byte(output[1]);
        buffer.push_byte(output[0]);
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

        buffer.push_byte(output[1]);
        buffer.push_byte(output[0]);
    }
}

impl AbiEncode for u8 {
    fn abi_encode(self, ref mut buffer: Buffer) {
        buffer.push_byte(self);
    }
}

// Encode str slice and str arrays

impl AbiEncode for str {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let len = self.len();
        buffer.push_u64(len);

        let ptr = self.as_ptr();

        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}

impl AbiEncode for str[0] {
    fn abi_encode(self, ref mut _buffer: Buffer) {}
}

// BEGIN STRARRAY_ENCODE
impl AbiEncode for str[1] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[2] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[3] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[4] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[5] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[6] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[7] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[8] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[9] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[10] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[11] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[12] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[13] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[14] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[15] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[16] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[17] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[18] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[19] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[20] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[21] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[22] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[23] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[24] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[25] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[26] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[27] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[28] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[29] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[30] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[31] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[32] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[33] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[34] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[35] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[36] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[37] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[38] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[39] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[40] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[41] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[42] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[43] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[44] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[45] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[46] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[47] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[48] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[49] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[50] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[51] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[52] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[53] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[54] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[55] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[56] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[57] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[58] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[59] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[60] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[61] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[62] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[63] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
impl AbiEncode for str[64] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);
        let len = s.len();
        let ptr = s.as_ptr();
        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
            i += 1;
        }
    }
}
// END STRARRAY_ENCODE

// Encode Arrays and Slices

impl AbiEncode for raw_slice {
    fn abi_encode(self, ref mut buffer: Buffer) {
        let len = self.number_of_bytes();
        buffer.push_u64(len);

        let ptr = self.ptr();

        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push_byte(byte);
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


impl AbiEncode for () {
    fn abi_encode(self, ref mut _buffer: Buffer) {}
}

// BEGIN TUPLES_ENCODE
impl<A> AbiEncode for (A, )
where
    A: AbiEncode,
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
        self.17.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
        self.17.abi_encode(buffer);
        self.18.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
        self.17.abi_encode(buffer);
        self.18.abi_encode(buffer);
        self.19.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
        self.17.abi_encode(buffer);
        self.18.abi_encode(buffer);
        self.19.abi_encode(buffer);
        self.20.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
        self.17.abi_encode(buffer);
        self.18.abi_encode(buffer);
        self.19.abi_encode(buffer);
        self.20.abi_encode(buffer);
        self.21.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
        self.17.abi_encode(buffer);
        self.18.abi_encode(buffer);
        self.19.abi_encode(buffer);
        self.20.abi_encode(buffer);
        self.21.abi_encode(buffer);
        self.22.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
        self.17.abi_encode(buffer);
        self.18.abi_encode(buffer);
        self.19.abi_encode(buffer);
        self.20.abi_encode(buffer);
        self.21.abi_encode(buffer);
        self.22.abi_encode(buffer);
        self.23.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
        self.17.abi_encode(buffer);
        self.18.abi_encode(buffer);
        self.19.abi_encode(buffer);
        self.20.abi_encode(buffer);
        self.21.abi_encode(buffer);
        self.22.abi_encode(buffer);
        self.23.abi_encode(buffer);
        self.24.abi_encode(buffer);
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
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.0.abi_encode(buffer);
        self.1.abi_encode(buffer);
        self.2.abi_encode(buffer);
        self.3.abi_encode(buffer);
        self.4.abi_encode(buffer);
        self.5.abi_encode(buffer);
        self.6.abi_encode(buffer);
        self.7.abi_encode(buffer);
        self.8.abi_encode(buffer);
        self.9.abi_encode(buffer);
        self.10.abi_encode(buffer);
        self.11.abi_encode(buffer);
        self.12.abi_encode(buffer);
        self.13.abi_encode(buffer);
        self.14.abi_encode(buffer);
        self.15.abi_encode(buffer);
        self.16.abi_encode(buffer);
        self.17.abi_encode(buffer);
        self.18.abi_encode(buffer);
        self.19.abi_encode(buffer);
        self.20.abi_encode(buffer);
        self.21.abi_encode(buffer);
        self.22.abi_encode(buffer);
        self.23.abi_encode(buffer);
        self.24.abi_encode(buffer);
        self.25.abi_encode(buffer);
    }
}
// END TUPLES_ENCODE

pub fn encode<T>(item: T) -> raw_slice
where
    T: AbiEncode,
{
    let mut buffer = Buffer::new();
    item.abi_encode(buffer);
    buffer.as_raw_slice()
}

pub fn abi_decode<T>(data: raw_slice) -> T
where
    T: AbiDecode,
{
    let mut buffer = BufferReader::from_parts(data.ptr(), data.len::<u8>());
    T::abi_decode(buffer)
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
        use ::primitive_conversions::*;
        let a = buffer.read::<u8>().as_u32();
        let b = buffer.read::<u8>().as_u32();
        let c = buffer.read::<u8>().as_u32();
        let d = buffer.read::<u8>().as_u32();
        (a << 24) | (b << 16) | (c << 8) | d
    }
}

impl AbiDecode for u16 {
    fn abi_decode(ref mut buffer: BufferReader) -> u16 {
        use ::primitive_conversions::*;
        let a = buffer.read::<u8>().as_u16();
        let b = buffer.read::<u8>().as_u16();
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

impl<T> AbiDecode for [T; 0]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut _buffer: BufferReader) -> [T; 0] {
        []
    }
}

impl<T> AbiDecode for [T; 1]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 1] {
        [T::abi_decode(buffer)]
    }
}

impl<T> AbiDecode for [T; 2]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 2] {
        [T::abi_decode(buffer), T::abi_decode(buffer)]
    }
}

impl<T> AbiDecode for [T; 3]
where
    T: AbiDecode,
{
    fn abi_decode(ref mut buffer: BufferReader) -> [T; 3] {
        [T::abi_decode(buffer), T::abi_decode(buffer), T::abi_decode(buffer)]
    }
}

impl<T> AbiDecode for [T; 4]
where
    T: AbiDecode,
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
    T: Eq + AbiEncode + AbiDecode,
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

#[test]
fn ok_abi_encoding() {
    // bool
    assert_encoding_and_decoding(false, [0u8]);
    assert_encoding_and_decoding(true, [1u8]);

    // numbers
    assert_encoding_and_decoding(0u8, [0u8; 1]);
    assert_encoding_and_decoding(255u8, [255u8; 1]);
    assert_encoding_and_decoding(0u16, [0u8; 2]);
    assert_encoding_and_decoding(65535u16, [255u8; 2]);
    assert_encoding_and_decoding(0u32, [0u8; 4]);
    assert_encoding_and_decoding(4294967295u32, [255u8; 4]);
    assert_encoding_and_decoding(0u64, [0u8; 8]);
    assert_encoding_and_decoding(18446744073709551615u64, [255u8; 8]);
    assert_encoding_and_decoding(
        0x0000000000000000000000000000000000000000000000000000000000000000u256,
        [0u8; 32],
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
}

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
