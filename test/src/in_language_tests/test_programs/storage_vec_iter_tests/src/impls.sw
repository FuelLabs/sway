library;

use std::ops::Eq;
use std::flags::{disable_panic_on_overflow, enable_panic_on_overflow};
use std::raw_slice::*;

pub trait TestInstance {
    /// Returns a [Vec] of `len` quasi-random elements
    /// of the type for which the trait is implemented.
    fn elements(len: u64) -> Vec<Self>;
}

impl TestInstance for bool {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            res.push(i % 3 == 0);
            i += 1;
        }
        res
    }
}

impl TestInstance for u8 {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(1);
        res.push(2);
        let mut i = 2;
        let _ = disable_panic_on_overflow();
        while i < len {
            res.push(res.get(i - 1).unwrap() + res.get(i - 2).unwrap());
            i += 1;
        }
        enable_panic_on_overflow();
        res
    }
}

impl TestInstance for u16 {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(1);
        res.push(2);
        let mut i = 2;
        let _ = disable_panic_on_overflow();
        while i < len {
            res.push(res.get(i - 1).unwrap() + res.get(i - 2).unwrap());
            i += 1;
        }
        enable_panic_on_overflow();
        res
    }
}

impl TestInstance for u32 {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(1);
        res.push(2);
        let mut i = 2;
        let _ = disable_panic_on_overflow();
        while i < len {
            res.push(res.get(i - 1).unwrap() + res.get(i - 2).unwrap());
            i += 1;
        }
        enable_panic_on_overflow();
        res
    }
}

impl TestInstance for u64 {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(1);
        res.push(2);
        let mut i = 2;
        let _ = disable_panic_on_overflow();
        while i < len {
            res.push(res.get(i - 1).unwrap() + res.get(i - 2).unwrap());
            i += 1;
        }
        enable_panic_on_overflow();
        res
    }
}

impl TestInstance for u256 {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256);
        let mut i = 1;
        while i < len {
            res.push(res.get(i - 1).unwrap() / i.into());
            i += 1;
        }
        res
    }
}

impl TestInstance for str {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            let string: str = match i % 3 {
                0 => "a1",
                1 => "b2c3",
                2 => "d4e5f6",
                _ => "unreachable",
            };
            res.push(string);
            i += 1;
        }
        res
    }
}

#[cfg(experimental_const_generics = false)]
impl PartialEq for str[6] {
    fn eq(self, other: Self) -> bool {
        let mut i = 0;
        while i < 6 {
            let ptr_self = __addr_of(self).add::<u8>(i);
            let ptr_other = __addr_of(other).add::<u8>(i);

            if ptr_self.read::<u8>() != ptr_other.read::<u8>() {
                return false;
            }

            i = i + 1;
        };

        true
    }
}
#[cfg(experimental_const_generics = false)]
impl Eq for str[6] {}

impl TestInstance for str[6] {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            let string: str[6] = match i % 3 {
                0 => __to_str_array("a1a1a1"),
                1 => __to_str_array("b2c3b2"),
                2 => __to_str_array("d4e5f6"),
                _ => __to_str_array("......"),
            };
            res.push(string);
            i += 1;
        }
        res
    }
}

#[cfg(experimental_const_generics = false)]
impl PartialEq for [u64; 2] {
    fn eq(self, other: Self) -> bool {
        self[0] == other[0] && self[1] == other[1]
    }
}
#[cfg(experimental_const_generics = false)]
impl Eq for [u64; 2] {}

impl TestInstance for [u64; 2] {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push([1, 1]);
        res.push([2, 2]);
        let mut i = 2;
        let _ = disable_panic_on_overflow();
        while i < len {
            let val = res.get(i - 1).unwrap()[0] + res.get(i - 2).unwrap()[0];
            res.push([val, val]);
            i += 1;
        }
        enable_panic_on_overflow();
        res
    }
}

pub struct Struct {
    x: u64,
}

impl PartialEq for Struct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}
impl Eq for Struct {}

impl TestInstance for Struct {
    fn elements(len: u64) -> Vec<Self> {
        let values = u64::elements(len);
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            res.push(Struct {
                x: values.get(i).unwrap(),
            });
            i += 1;
        }
        res
    }
}

pub struct EmptyStruct {}

impl PartialEq for EmptyStruct {
    fn eq(self, other: Self) -> bool {
        true
    }
}
impl Eq for EmptyStruct {}

impl TestInstance for EmptyStruct {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            res.push(EmptyStruct {});
            i += 1;
        }
        res
    }
}

pub enum Enum {
    A: u64,
}

impl PartialEq for Enum {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (Enum::A(l), Enum::A(r)) => l == r,
        }
    }
}
impl Eq for Enum {}

impl TestInstance for Enum {
    fn elements(len: u64) -> Vec<Self> {
        let values = u64::elements(len);
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            res.push(Enum::A(values.get(i).unwrap()));
            i += 1;
        }
        res
    }
}

impl TestInstance for (u8, u32) {
    fn elements(len: u64) -> Vec<Self> {
        let u8_values = u8::elements(len);
        let u32_values = u32::elements(len);
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            res.push((u8_values.get(i).unwrap(), u32_values.get(i).unwrap()));
            i += 1;
        }
        res
    }
}

impl TestInstance for b256 {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(
            0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256
                .as_b256(),
        );
        let mut i = 1;
        while i < len {
            res.push((res.get(i - 1).unwrap().as_u256() / i.into()).as_b256());
            i += 1;
        }
        res
    }
}

pub struct RawPtrNewtype {
    ptr: raw_ptr,
}

impl TestInstance for RawPtrNewtype {
    fn elements(len: u64) -> Vec<Self> {
        let values = u8::elements(len);
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            let null_ptr = asm() {
                zero: raw_ptr
            };
            res.push(RawPtrNewtype {
                ptr: null_ptr.add::<u64>(values.get(i).unwrap().as_u64())
            });
            i += 1;
        }
        res
    }
}

impl AbiEncode for RawPtrNewtype {
    fn abi_encode(self, buffer: Buffer) -> Buffer {
        let ptr_as_u64 = asm(p: self.ptr) {
            p: u64
        };
        ptr_as_u64.abi_encode(buffer)
    }
}

impl PartialEq for RawPtrNewtype {
    fn eq(self, other: Self) -> bool {
        self.ptr == other.ptr
    }
}
impl Eq for RawPtrNewtype {}

impl TestInstance for raw_slice {
    fn elements(len: u64) -> Vec<Self> {
        let raw_ptr_values = RawPtrNewtype::elements(len);
        let mut ptr_values = Vec::<raw_ptr>::new();
        for raw_ptr in raw_ptr_values.iter() {
            ptr_values.push(raw_ptr.ptr);
        }

        let len_values = u8::elements(len);
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            res.push(raw_slice::from_parts::<u64>(
                ptr_values
                    .get(i)
                    .unwrap(),
                len_values
                    .get(i)
                    .unwrap()
                    .as_u64(),
            ));
            i += 1;
        }
        res
    }
}

impl TestInstance for () {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            res.push(());
            i += 1;
        }
        res
    }
}

impl TestInstance for [u64; 0] {
    fn elements(len: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < len {
            res.push([]);
            i += 1;
        }
        res
    }
}

#[cfg(experimental_const_generics = false)]
impl PartialEq for [u64; 0] {
    fn eq(self, other: Self) -> bool {
        true
    }
}
#[cfg(experimental_const_generics = false)]
impl Eq for [u64; 0] {}
