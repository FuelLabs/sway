//! This library contains types and traits meant to be used
//! in tests that need to test functionality across different,
//! representative, Sway types.
library;

use std::ops::Eq;
use std::flags::{disable_panic_on_overflow, enable_panic_on_overflow};
use std::raw_slice::*;

#[allow(dead_code)] // TODO-DCA: Remove this `allow` once https://github.com/FuelLabs/sway/issues/7462 is fixed.
pub fn assert_non_zero_sized_type<T>() {
    if __size_of::<T>() == 0 {
        panic "Type must be a non-zero-sized type."
    }
}

pub type ArrayU8Len2 = [u8; 2];
pub type ArrayU64Len3 = [u64; 3];
pub type ArrayNestedArrayU8Len2Len3 = [ArrayU8Len2; 3];

pub trait TestInstance {
    /// Returns a default instance of `Self`.
    fn default() -> Self;

    /// Returns a [Vec] of `count` quasi-random instances of `Self`.
    fn instances(count: u64) -> Vec<Self>;
} {
    /// Returns a [Vec] of `count` default instances of `Self`.
    fn default_instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push(Self::default());
            i += 1;
        }
        res
    }
}

impl TestInstance for bool {
    fn default() -> Self {
        false
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push(i % 3 == 0);
            i += 1;
        }
        res
    }
}

impl TestInstance for u8 {
    fn default() -> Self {
        0
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(1);
        res.push(2);
        let mut i = 2;
        let _ = disable_panic_on_overflow();
        while i < count {
            res.push(res.get(i - 1).unwrap() + res.get(i - 2).unwrap());
            i += 1;
        }
        enable_panic_on_overflow();
        res
    }
}

impl TestInstance for u16 {
    fn default() -> Self {
        0
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(1);
        res.push(2);
        let mut i = 2;
        let _ = disable_panic_on_overflow();
        while i < count {
            res.push(res.get(i - 1).unwrap() + res.get(i - 2).unwrap());
            i += 1;
        }
        enable_panic_on_overflow();
        res
    }
}

impl TestInstance for u32 {
    fn default() -> Self {
        0
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(1);
        res.push(2);
        let mut i = 2;
        let _ = disable_panic_on_overflow();
        while i < count {
            res.push(res.get(i - 1).unwrap() + res.get(i - 2).unwrap());
            i += 1;
        }
        enable_panic_on_overflow();
        res
    }
}

impl TestInstance for u64 {
    fn default() -> Self {
        0
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(1);
        res.push(2);
        let mut i = 2;
        let _ = disable_panic_on_overflow();
        while i < count {
            res.push(res.get(i - 1).unwrap() + res.get(i - 2).unwrap());
            i += 1;
        }
        enable_panic_on_overflow();
        res
    }
}

impl TestInstance for u256 {
    fn default() -> Self {
        0u256
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256);
        let mut i = 1;
        while i < count {
            res.push(res.get(i - 1).unwrap() / i.into());
            i += 1;
        }
        res
    }
}

impl TestInstance for str {
    fn default() -> Self {
        ""
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
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

impl TestInstance for str[2] {
    fn default() -> Self {
        __to_str_array("..")
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            let string: str[2] = match i % 3 {
                0 => __to_str_array("a1"),
                1 => __to_str_array("b2"),
                2 => __to_str_array("d4"),
                _ => __to_str_array(".."),
            };
            res.push(string);
            i += 1;
        }
        res
    }
}

impl TestInstance for str[6] {
    fn default() -> Self {
        __to_str_array("......")
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
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

impl TestInstance for str[8] {
    fn default() -> Self {
        __to_str_array("........")
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            let string: str[8] = match i % 3 {
                0 => __to_str_array("a1a1a1a1"),
                1 => __to_str_array("b2c3b2c3"),
                2 => __to_str_array("d4e5f6g7"),
                _ => __to_str_array("........"),
            };
            res.push(string);
            i += 1;
        }
        res
    }
}

impl TestInstance for str[12] {
    fn default() -> Self {
        __to_str_array("............")
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            let string: str[12] = match i % 3 {
                0 => __to_str_array("a1a1a1a1a1a1"),
                1 => __to_str_array("b2c3b2c3b2c3"),
                2 => __to_str_array("d4e5f6g7h8i9"),
                _ => __to_str_array("............"),
            };
            res.push(string);
            i += 1;
        }
        res
    }
}

impl<T, const N: u64> TestInstance for [T; N]
where
    T: TestInstance,
{
    fn default() -> Self {
        [T::default(); N]
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let instances = T::instances(count);
        let mut i = 0;
        while i < count {
            let mut array = [T::default(); N];
            let mut j = 0;
            while j < N {
                array[j] = instances.get(i).unwrap();
                j += 1;
            }
            res.push(array);
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
    fn default() -> Self {
        EmptyStruct {}
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push(EmptyStruct {});
            i += 1;
        }
        res
    }
}

pub enum EnumSingleU8 {
    A: u8,
}

impl PartialEq for EnumSingleU8 {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (EnumSingleU8::A(l), EnumSingleU8::A(r)) => l == r,
        }
    }
}
impl Eq for EnumSingleU8 {}

impl TestInstance for EnumSingleU8 {
    fn default() -> Self {
        EnumSingleU8::A(u8::default())
    }

    fn instances(count: u64) -> Vec<Self> {
        let values = u8::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push(EnumSingleU8::A(values.get(i).unwrap()));
            i += 1;
        }
        res
    }
}

pub enum EnumSingleU64 {
    A: u64,
}

impl PartialEq for EnumSingleU64 {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (EnumSingleU64::A(l), EnumSingleU64::A(r)) => l == r,
        }
    }
}
impl Eq for EnumSingleU64 {}

impl TestInstance for EnumSingleU64 {
    fn default() -> Self {
        EnumSingleU64::A(u64::default())
    }

    fn instances(count: u64) -> Vec<Self> {
        let values = u64::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push(EnumSingleU64::A(values.get(i).unwrap()));
            i += 1;
        }
        res
    }
}

pub enum EnumSingleBool {
    A: bool,
}

impl PartialEq for EnumSingleBool {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (EnumSingleBool::A(l), EnumSingleBool::A(r)) => l == r,
        }
    }
}
impl Eq for EnumSingleBool {}

impl TestInstance for EnumSingleBool {
    fn default() -> Self {
        EnumSingleBool::A(bool::default())
    }

    fn instances(count: u64) -> Vec<Self> {
        let values = bool::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push(EnumSingleBool::A(values.get(i).unwrap()));
            i += 1;
        }
        res
    }
}

pub enum EnumMultiUnits {
    A: (),
    B: (),
    C: (),
}

impl PartialEq for EnumMultiUnits {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (EnumMultiUnits::A, EnumMultiUnits::A) => true,
            (EnumMultiUnits::B, EnumMultiUnits::B) => true,
            (EnumMultiUnits::C, EnumMultiUnits::C) => true,
            _ => false,
        }
    }
}
impl Eq for EnumMultiUnits {}

impl TestInstance for EnumMultiUnits {
    fn default() -> Self {
        EnumMultiUnits::A
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            match i % 3 {
                0 => res.push(EnumMultiUnits::A),
                1 => res.push(EnumMultiUnits::B),
                _ => res.push(EnumMultiUnits::C),
            };
            i += 1;
        }
        res
    }
}

pub enum EnumMultiOneByte {
    A: bool,
    B: u8,
    C: (),
}

impl PartialEq for EnumMultiOneByte {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (EnumMultiOneByte::A(l), EnumMultiOneByte::A(r)) => l == r,
            (EnumMultiOneByte::B(l), EnumMultiOneByte::B(r)) => l == r,
            (EnumMultiOneByte::C, EnumMultiOneByte::C) => true,
            _ => false,
        }
    }
}
impl Eq for EnumMultiOneByte {}

impl TestInstance for EnumMultiOneByte {
    fn default() -> Self {
        EnumMultiOneByte::A(bool::default())
    }

    fn instances(count: u64) -> Vec<Self> {
        let bool_values = bool::instances(count);
        let u8_values = u8::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            match i % 3 {
                0 => res.push(EnumMultiOneByte::A(bool_values.get(i).unwrap())),
                1 => res.push(EnumMultiOneByte::B(u8_values.get(i).unwrap())),
                _ => res.push(EnumMultiOneByte::C),
            };
            i += 1;
        }
        res
    }
}

pub enum EnumU8AndU64 {
    A: u8,
    B: u64,
}

impl PartialEq for EnumU8AndU64 {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (EnumU8AndU64::A(l), EnumU8AndU64::A(r)) => l == r,
            (EnumU8AndU64::B(l), EnumU8AndU64::B(r)) => l == r,
            _ => false,
        }
    }
}
impl Eq for EnumU8AndU64 {}

impl TestInstance for EnumU8AndU64 {
    fn default() -> Self {
        EnumU8AndU64::A(u8::default())
    }

    fn instances(count: u64) -> Vec<Self> {
        let u8_values = u8::instances(count);
        let u64_values = u64::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            match i % 2 {
                0 => res.push(EnumU8AndU64::A(u8_values.get(i).unwrap())),
                _ => res.push(EnumU8AndU64::B(u64_values.get(i).unwrap())),
            };
            i += 1;
        }
        res
    }
}

pub enum EnumQuadSlotSize {
    A: u8,
    B: (u64, u64, u64),
}

impl PartialEq for EnumQuadSlotSize {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (EnumQuadSlotSize::A(l), EnumQuadSlotSize::A(r)) => l == r,
            (EnumQuadSlotSize::B(l), EnumQuadSlotSize::B(r)) => {
                l.0 == r.0 && l.1 == r.1 && l.2 == r.2
            }
            _ => false,
        }
    }
}
impl Eq for EnumQuadSlotSize {}

impl TestInstance for EnumQuadSlotSize {
    fn default() -> Self {
        EnumQuadSlotSize::A(u8::default())
    }

    fn instances(count: u64) -> Vec<Self> {
        let u8_values = u8::instances(count);
        let u64_values = u64::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            match i % 2 {
                0 => res.push(EnumQuadSlotSize::A(u8_values.get(i).unwrap())),
                _ => res.push(EnumQuadSlotSize::B((
                    u64_values.get(i).unwrap(),
                    u64_values.get((i + 1) % count).unwrap(),
                    u64_values.get((i + 2) % count).unwrap(),
                ))),
            };
            i += 1;
        }
        res
    }
}

pub enum EnumLargerThanQuadSlot {
    A: u8,
    B: (u64, u64, u64, u64, u64),
}

impl PartialEq for EnumLargerThanQuadSlot {
    fn eq(self, other: Self) -> bool {
        match (self, other) {
            (EnumLargerThanQuadSlot::A(l), EnumLargerThanQuadSlot::A(r)) => l == r,
            (EnumLargerThanQuadSlot::B(l), EnumLargerThanQuadSlot::B(r)) => {
                l.0 == r.0 && l.1 == r.1 && l.2 == r.2 && l.3 == r.3 && l.4 == r.4
            }
            _ => false,
        }
    }
}
impl Eq for EnumLargerThanQuadSlot {}

impl TestInstance for EnumLargerThanQuadSlot {
    fn default() -> Self {
        EnumLargerThanQuadSlot::A(u8::default())
    }

    fn instances(count: u64) -> Vec<Self> {
        let u8_values = u8::instances(count);
        let u64_values = u64::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            match i % 2 {
                0 => res.push(EnumLargerThanQuadSlot::A(u8_values.get(i).unwrap())),
                _ => res.push(EnumLargerThanQuadSlot::B((
                    u64_values.get(i).unwrap(),
                    u64_values.get((i + 1) % count).unwrap(),
                    u64_values.get((i + 2) % count).unwrap(),
                    u64_values.get((i + 3) % count).unwrap(),
                    u64_values.get((i + 4) % count).unwrap(),
                ))),
            };
            i += 1;
        }
        res
    }
}

pub struct StructA {
    f_bool: bool,
    f_u8: u8,
    f_u16: u16,
    f_u32: u32,
    f_u64: u64,
    f_u256: u256,
}

impl PartialEq for StructA {
    fn eq(self, other: Self) -> bool {
        self.f_bool == other.f_bool
            && self.f_u8 == other.f_u8
            && self.f_u16 == other.f_u16
            && self.f_u32 == other.f_u32
            && self.f_u64 == other.f_u64
            && self.f_u256 == other.f_u256
    }
}
impl Eq for StructA {}

impl TestInstance for StructA {
    fn default() -> Self {
        StructA {
            f_bool: bool::default(),
            f_u8: u8::default(),
            f_u16: u16::default(),
            f_u32: u32::default(),
            f_u64: u64::default(),
            f_u256: u256::default(),
        }
    }

    fn instances(count: u64) -> Vec<Self> {
        let bool_values = bool::instances(count);
        let u8_values = u8::instances(count);
        let u16_values = u16::instances(count);
        let u32_values = u32::instances(count);
        let u64_values = u64::instances(count);
        let u256_values = u256::instances(count);

        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push(StructA {
                f_bool: bool_values.get(i).unwrap(),
                f_u8: u8_values.get(i).unwrap(),
                f_u16: u16_values.get(i).unwrap(),
                f_u32: u32_values.get(i).unwrap(),
                f_u64: u64_values.get(i).unwrap(),
                f_u256: u256_values.get(i).unwrap(),
            });
            i += 1;
        }
        res
    }
}

pub struct StructB {
    f_struct_a: StructA,
    f_tuple: (u8, u16, u32, u64, u256),
    // TODO-IG!: Add link to GitHub issue.
    // f_array_u8_len_2: ArrayU8Len2,
    // f_array_nested_array_u8_len_2_len_3: ArrayNestedArrayU8Len2Len3,
    f_array_u8_len_2: [u8; 2],
    f_array_nested_array_u8_len_2_len_3: [[u8; 2]; 3],
}

pub type ArrayStructBLen2 = [StructB; 2];

impl PartialEq for StructB {
    fn eq(self, other: Self) -> bool {
        self.f_struct_a == other.f_struct_a
            && self.f_tuple.0 == other.f_tuple.0
            && self.f_tuple.1 == other.f_tuple.1
            && self.f_tuple.2 == other.f_tuple.2
            && self.f_tuple.3 == other.f_tuple.3
            && self.f_tuple.4 == other.f_tuple.4
            && self.f_array_u8_len_2 == other.f_array_u8_len_2
            && self.f_array_nested_array_u8_len_2_len_3
                == other.f_array_nested_array_u8_len_2_len_3
    }
}
impl Eq for StructB {}

impl TestInstance for StructB {
    fn default() -> Self {
        StructB {
            f_struct_a: StructA::default(),
            f_tuple: (
                u8::default(),
                u16::default(),
                u32::default(),
                u64::default(),
                u256::default(),
            ),
            // TODO-IG!: Add link to GitHub issue.
            // f_array_u8_len_2: ArrayU8Len2::default(count),
            // f_array_nested_array_u8_len_2_len_3: ArrayNestedArrayU8Len2Len3::default(count),
            f_array_u8_len_2: [u8::default(); 2],
            f_array_nested_array_u8_len_2_len_3: [[u8::default(); 2]; 3],
        }
    }

    fn instances(count: u64) -> Vec<Self> {
        let struct_a_values = StructA::instances(count);
        let u8_values = u8::instances(count);
        let u16_values = u16::instances(count);
        let u32_values = u32::instances(count);
        let u64_values = u64::instances(count);
        let u256_values = u256::instances(count);
        // TODO-IG!: Add link to GitHub issue.
        // let array_u8_len_2_values = ArrayU8Len2::instances(count);
        // let array_nested_array_u8_len_2_len_3_values = ArrayNestedArrayU8Len2Len3::instances(count);

        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push(StructB {
                f_struct_a: struct_a_values.get(i).unwrap(),
                f_tuple: (
                    u8_values.get(i).unwrap(),
                    u16_values.get(i).unwrap(),
                    u32_values.get(i).unwrap(),
                    u64_values.get(i).unwrap(),
                    u256_values.get(i).unwrap(),
                ),
                f_array_u8_len_2: [u8_values.get(i).unwrap(); 2],
                f_array_nested_array_u8_len_2_len_3: [[u8_values.get(i).unwrap(); 2]; 3],
                // TODO-IG!: Add link to GitHub issue.
                // f_array_u8_len_2: array_u8_len_2_values.get(i).unwrap(),
                // f_array_nested_array_u8_len_2_len_3:
                //     array_nested_array_u8_len_2_len_3_values.get(i).unwrap(),
            });
            i += 1;
        }
        res
    }
}

impl TestInstance for (u8, u32) {
    fn default() -> Self {
        (u8::default(), u32::default())
    }

    fn instances(count: u64) -> Vec<Self> {
        let u8_values = u8::instances(count);
        let u32_values = u32::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push((u8_values.get(i).unwrap(), u32_values.get(i).unwrap()));
            i += 1;
        }
        res
    }
}

impl TestInstance for b256 {
    fn default() -> Self {
        0u256.as_b256()
    }

    fn instances(count: u64) -> Vec<Self> {
        let mut res = Vec::new();
        res.push(
            0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20u256
                .as_b256(),
        );
        let mut i = 1;
        while i < count {
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
    fn default() -> Self {
        let null_ptr = asm() {
            zero: raw_ptr
        };
        RawPtrNewtype { ptr: null_ptr }
    }

    fn instances(count: u64) -> Vec<Self> {
        let values = u8::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
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
    fn is_encode_trivial() -> bool { false }
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
    fn default() -> Self {
        raw_slice::from_parts::<u64>(RawPtrNewtype::default().ptr, 0)
    }

    fn instances(count: u64) -> Vec<Self> {
        let raw_ptr_values = RawPtrNewtype::instances(count);
        let mut ptr_values = Vec::<raw_ptr>::new();
        for raw_ptr in raw_ptr_values.iter() {
            ptr_values.push(raw_ptr.ptr);
        }

        let len_values = u8::instances(count);
        let mut res = Vec::new();
        let mut i = 0;
        while i < count {
            res.push(raw_slice::from_parts::<u64>(
                ptr_values
                    .get(i)
                    .unwrap(),
                len_values
                    .get(i)
                    .unwrap()
                    .as_u64()
            ));
            i += 1;
        }
        res
    }
}
