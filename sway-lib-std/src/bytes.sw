library bytes;

use ::{alloc::alloc, vec::Vec};
use ::assert::assert;

pub struct Bytes {
    ptr: u64,
    len: u64,
    cap: u64,
}

impl Bytes {

    // fn new(len: u64) -> Bytes {
    //     let i = len;
    //     while i > 0 {
    //         bytes.push_byte(0u8)
    //     }
    // }
    fn push_byte(ref mut self, item: u8) {
    // if ptr + len > cap then we need to alloc more memory
        if self.ptr + self.len > self.ptr + self.cap {
            self.cap = self.cap * 2;
            self.ptr = asm(r1: alloc(self.cap)) {
                r1: u64
            };
        }

        asm(r1: self.ptr + self.len, r2: item) {
            sb r1 r2 i0;
        }
        self.len = self.len + 1;
    }


//   fn pop_byte(ref mut self) -> u8 {
//       asm(r1: self.ptr + self.len - 1, r2) {
//         lb r2 r1 i0;
//         r2: u8
//       }
//   }
  // can use From trait when generic traits are in
    fn from_vec_u8(raw: Vec<u8>) -> Self {
    // TODO
        Bytes { ptr: 0, len: 0, cap: 0}
    }

//   fn into::<T>(self) -> Vec<u8> {
//   }
}

#[test()]
fn test_bytes_literal_intantiation() {
   let bytes =  Bytes {
        ptr: 11,
        len: 42,
        cap: 42
    };
    assert(bytes.ptr == 11);
    assert(bytes.len == 42);
    assert(bytes.cap == 42);
}
