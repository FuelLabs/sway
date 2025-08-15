script;

// these should use MCLI

// u8
#[inline(never)]
fn array_repeat_zero_small_u8() -> [u8; 5] {
    [0u8; 5]
}
#[inline(never)]
fn array_repeat_zero_big_u8() -> [u8; 25] {
    [0u8; 25]
}

// u16
#[inline(never)]
fn array_repeat_zero_small_u16() -> [u16; 5] {
    [0u16; 5]
}
#[inline(never)]
fn array_repeat_zero_big_u16() -> [u16; 25] {
    [0u16; 25]
}

// u32
#[inline(never)]
fn array_repeat_zero_small_u32() -> [u32; 5] {
    [0u32; 5]
}
#[inline(never)]
fn array_repeat_zero_big_u32() -> [u32; 25] {
    [0u32; 25]
}

// u64
#[inline(never)]
fn array_repeat_zero_small_u64() -> [u64; 5] {
    [0u64; 5]
}
#[inline(never)]
fn array_repeat_zero_big_u64() -> [u64; 25] {
    [0u64; 25]
}

// u256
#[inline(never)]
fn array_repeat_zero_small_u256() -> [u256; 5] {
    [0x0000000000000000000000000000000000000000000000000000000000000000u256; 5]
}
#[inline(never)]
fn array_repeat_zero_big_u256() -> [u256; 25] {
    [0x0000000000000000000000000000000000000000000000000000000000000000u256; 25]
}

// b256
#[inline(never)]
fn array_repeat_zero_small_b256() -> [b256; 5] {
    [0x0000000000000000000000000000000000000000000000000000000000000000; 5]
}
#[inline(never)]
fn array_repeat_zero_big_b256() -> [b256; 25] {
    [0x0000000000000000000000000000000000000000000000000000000000000000; 25]
}

// bool
#[inline(never)]
fn array_repeat_zero_small_bool() -> [bool; 5] {
    [false; 5]
}
#[inline(never)]
fn array_repeat_zero_big_bool() -> [bool; 25] {
    [false; 25]
}

// This will store each item individually
#[inline(never)]
fn small_array_repeat() -> [bool; 5] {
    [true; 5]
}

// This will use a loop
#[inline(never)]
fn big_array_repeat() -> [bool; 25] {
    [true; 25]
}

// This array is bigger than 2^18, se we need to use MCL
//  2^18 = 262144
#[inline(never)]
fn u8_array_bigger_than_18_bits() -> [u8; 262145] {
    [0u8; 262145]
}

fn main() {
    let _ = array_repeat_zero_small_u8();
    let _ = array_repeat_zero_small_u16();
    let _ = array_repeat_zero_small_u32();
    let _ = array_repeat_zero_small_u64();
    let _ = array_repeat_zero_small_u256();
    let _ = array_repeat_zero_small_b256();
    let _ = array_repeat_zero_small_bool();

    let _ = array_repeat_zero_big_u8();
    let _ = array_repeat_zero_big_u16();
    let _ = array_repeat_zero_big_u32();
    let _ = array_repeat_zero_big_u64();
    let _ = array_repeat_zero_big_u256();
    let _ = array_repeat_zero_big_b256();
    let _ = array_repeat_zero_big_bool();

    let _ = small_array_repeat();
    let _ = big_array_repeat();

    let _ = u8_array_bigger_than_18_bits();
}

trait IsZero { fn is_zero(self) -> bool; }

impl IsZero for bool { fn is_zero(self) -> bool { self == false }}
impl IsZero for u8 { fn is_zero(self) -> bool { self == 0 }}
impl IsZero for u16 { fn is_zero(self) -> bool { self == 0 }}
impl IsZero for u32 { fn is_zero(self) -> bool { self == 0 }}
impl IsZero for u64 { fn is_zero(self) -> bool { self == 0 }}
impl IsZero for u256 { fn is_zero(self) -> bool { self == 0x0000000000000000000000000000000000000000000000000000000000000000u256 }}
impl IsZero for b256 { fn is_zero(self) -> bool { self == 0x0000000000000000000000000000000000000000000000000000000000000000 }}

fn is_all_zero<T, const N: u64>(array: [T; N], n: u64) where T: IsZero {
    let mut i = 0;
    while i < n {
        assert(array[i].is_zero());
        i += 1;
    }
}

#[test]
fn test_array_repeat_zero() {
    is_all_zero(array_repeat_zero_small_u8(), 5);
    is_all_zero(array_repeat_zero_small_u16(), 5);
    is_all_zero(array_repeat_zero_small_u32(), 5);
    is_all_zero(array_repeat_zero_small_u64(), 5);
    is_all_zero(array_repeat_zero_small_u256(), 5);
    is_all_zero(array_repeat_zero_small_b256(), 5);
    is_all_zero(array_repeat_zero_small_bool(), 5);

    is_all_zero(array_repeat_zero_big_u8(), 25);
    is_all_zero(array_repeat_zero_big_u16(), 25);
    is_all_zero(array_repeat_zero_big_u32(), 25);
    is_all_zero(array_repeat_zero_big_u64(), 25);
    is_all_zero(array_repeat_zero_big_u256(), 25);
    is_all_zero(array_repeat_zero_big_b256(), 25);
    is_all_zero(array_repeat_zero_big_bool(), 25);

    is_all_zero(u8_array_bigger_than_18_bits(), 262145);
}