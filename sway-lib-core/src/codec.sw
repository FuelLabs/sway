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

pub trait AbiEncode {
    fn abi_encode(self, ref mut buffer: Buffer);
}

impl AbiEncode for () {
    fn abi_encode(self, ref mut _buffer: Buffer) {}
}

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

impl AbiEncode for bool {
    fn abi_encode(self, ref mut buffer: Buffer) {
        buffer.push(self);
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

// str arrays

impl AbiEncode for str[0] {
    fn abi_encode(self, ref mut _buffer: Buffer) {}
}

impl AbiEncode for str[1] {
    fn abi_encode(self, ref mut buffer: Buffer) {
        use ::str::*;
        let s = from_str_array(self);

        let len = s.len();
        let ptr = s.as_ptr();

        let mut i = 0;
        while i < len {
            let byte = ptr.add::<u8>(i).read::<u8>();
            buffer.push(byte);
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
            buffer.push(byte);
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
            buffer.push(byte);
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
            buffer.push(byte);
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
            buffer.push(byte);
            i += 1;
        }
    }
}

// arrays

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

// Tuples

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

pub fn encode<T>(item: T) -> raw_slice
where
    T: AbiEncode,
{
    let mut buffer = Buffer::new();
    item.abi_encode(buffer);
    buffer.as_raw_slice()
}

#[test]
fn ok_encode() {
    let _ = encode(true);

    let _ = encode(0u8);
    let _ = encode(0u16);
    let _ = encode(0u32);
    let _ = encode(0u64);
    let _ = encode(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFu256);

    // b256
    let _ = encode(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
}
