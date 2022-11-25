library bytes;

use ::{alloc::alloc, vec::Vec};
use ::assert::assert;
use ::option::Option;

pub struct Bytes {
    ptr: u64,
    len: u64,
    cap: u64,
}

impl Bytes {
    pub fn new() -> Self {
        Bytes {
            ptr: asm(r1: alloc::<u8>(0)) { r1: u64 },
            len: 0,
            cap: 0,
        }
    }

    pub fn with_capacity(capacity: u64) -> Self {
        Bytes {
            ptr: asm(r1: alloc::<u8>(capacity)) { r1: u64 },
            len: 0,
            cap: capacity,
        }
    }
}

impl Bytes {
    pub fn push_byte(ref mut self, item: u8) {
        // if ptr + len > cap then we need to alloc more memory
        if self.ptr + self.len > self.ptr + self.cap {
            self.cap = self.cap * 2;
            self.ptr = asm(r1: alloc::<u8>(self.cap)) { r1: u64 };
        }

        asm(r1: self.ptr + self.len, r2: item) {
            sb r1 r2 i0;
        }
        self.len = self.len + 1;
    }
}

// Need to use seperate impl blocks for now: https://github.com/FuelLabs/sway/issues/1548
impl Bytes {
    // can use From trait when generic traits are in
    pub fn from_vec_u8(ref mut raw: Vec<u8>) -> Self {
        let mut bytes = Bytes::new();
        let mut i = 0;
        let length = raw.len();

        while i < length {
            // @review unsure the following unwrap is safe.
            bytes.push_byte(raw.get(i).unwrap());
            bytes.len += 1;
            i += 1;
        };

        bytes
    }
}


#[test()]
fn test_from_vec_u8() {
    let mut vec = Vec::new();
    vec.push(11u8);
    vec.push(42u8);
    vec.push(69u8);
    let bytes = Bytes::from_vec_u8(vec);
    assert(bytes.len == 3);
}
