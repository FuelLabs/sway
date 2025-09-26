// This test will check if array initialization is correct. That means:
// - mcli for zero initialized arrays when the array size in bytes allows it;
// - mcl for zero initialized arrays when the array size in bytes is too big;
// - store each item individiually when the array is not zero initialized, but its length is small;
// - initialize the array using a loop for all other cases.
script;

// u8
#[inline(never)]
fn array_fn_should_use_mcli_small_u8() -> [u8; 5] {
    [0u8; 5]
}
#[inline(never)]
fn array_fn_should_use_mcli_big_u8() -> [u8; 25] {
    [0u8; 25]
}

// u16
#[inline(never)]
fn array_fn_should_use_mcli_small_u16() -> [u16; 5] {
    [0u16; 5]
}
#[inline(never)]
fn array_fn_should_use_mcli_big_u16() -> [u16; 25] {
    [0u16; 25]
}

// u32
#[inline(never)]
fn array_fn_should_use_mcli_small_u32() -> [u32; 5] {
    [0u32; 5]
}
#[inline(never)]
fn array_fn_should_use_mcli_big_u32() -> [u32; 25] {
    [0u32; 25]
}

// u64
#[inline(never)]
fn array_fn_should_use_mcli_small_u64() -> [u64; 5] {
    [0u64; 5]
}
#[inline(never)]
fn array_fn_should_use_mcli_big_u64() -> [u64; 25] {
    [0u64; 25]
}

// u256
#[inline(never)]
fn array_fn_should_use_mcli_small_u256() -> [u256; 5] {
    [0x0000000000000000000000000000000000000000000000000000000000000000u256; 5]
}
#[inline(never)]
fn array_fn_should_use_mcli_big_u256() -> [u256; 25] {
    [0x0000000000000000000000000000000000000000000000000000000000000000u256; 25]
}

// b256
#[inline(never)]
fn array_fn_should_use_mcli_small_b256() -> [b256; 5] {
    [0x0000000000000000000000000000000000000000000000000000000000000000; 5]
}
#[inline(never)]
fn array_fn_should_use_mcli_big_b256() -> [b256; 25] {
    [0x0000000000000000000000000000000000000000000000000000000000000000; 25]
}

// bool
#[inline(never)]
fn array_fn_should_use_mcli_small_bool() -> [bool; 5] {
    [false; 5]
}
#[inline(never)]
fn array_fn_should_use_mcli_big_bool() -> [bool; 25] {
    [false; 25]
}

#[inline(never)]
fn array_fn_should_store_each_item_bool() -> [bool; 5] {
    [true; 5]
}

#[inline(never)]
fn array_fn_should_loop_bool() -> [bool; 25] {
    [true; 25]
}

// This array is bigger than 2^18, se we need to use MCL
//  2^18 = 262144
#[inline(never)]
fn array_fn_should_use_mcl_u8() -> [u8; 262145] {
    [0u8; 262145]
}

// Test arrays length that are const declarations

const GLOBAL_A: u64 = 1;
#[inline(never)]
fn arrays_with_const_length() {
    const A = 1;
    const B = A + 1;

    let _ = [0u64; A];
    let _: [u64; A] = [0u64; A];
    let _ = [0u64; GLOBAL_A];

    let _: [u64; A] = [0u64];

    let _ = [0u64; B];
    let _: [u64; B] = [0u64, 1u64];
}

fn main() {
    let _ = array_fn_should_use_mcli_small_u8();
    let _ = array_fn_should_use_mcli_small_u16();
    let _ = array_fn_should_use_mcli_small_u32();
    let _ = array_fn_should_use_mcli_small_u64();
    let _ = array_fn_should_use_mcli_small_u256();
    let _ = array_fn_should_use_mcli_small_b256();
    let _ = array_fn_should_use_mcli_small_bool();

    let _ = array_fn_should_use_mcli_big_u8();
    let _ = array_fn_should_use_mcli_big_u16();
    let _ = array_fn_should_use_mcli_big_u32();
    let _ = array_fn_should_use_mcli_big_u64();
    let _ = array_fn_should_use_mcli_big_u256();
    let _ = array_fn_should_use_mcli_big_b256();
    let _ = array_fn_should_use_mcli_big_bool();

    let _ = array_fn_should_store_each_item_bool();
    let _ = array_fn_should_loop_bool();

    let _ = array_fn_should_use_mcl_u8();

    let _ = arrays_with_const_length();

    // Array decode
    let array: [u8; 1] = decode_array();
    assert_eq(array[0], 255u8);
}

#[inline(never)]
fn decode_array() -> [u8; 1] {
    let s: raw_slice = to_slice([255u8]);
    abi_decode::<[u8; 1]>(s)
}

#[inline(never)]
fn to_slice<T>(array: T) -> raw_slice {
    let len = __size_of::<T>();
    raw_slice::from_parts::<u8>(__addr_of(array), len)
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
fn test() {
    is_all_zero(array_fn_should_use_mcli_small_u8(), 5);
    is_all_zero(array_fn_should_use_mcli_small_u16(), 5);
    is_all_zero(array_fn_should_use_mcli_small_u32(), 5);
    is_all_zero(array_fn_should_use_mcli_small_u64(), 5);
    is_all_zero(array_fn_should_use_mcli_small_u256(), 5);
    is_all_zero(array_fn_should_use_mcli_small_b256(), 5);
    is_all_zero(array_fn_should_use_mcli_small_bool(), 5);

    is_all_zero(array_fn_should_use_mcli_big_u8(), 25);
    is_all_zero(array_fn_should_use_mcli_big_u16(), 25);
    is_all_zero(array_fn_should_use_mcli_big_u32(), 25);
    is_all_zero(array_fn_should_use_mcli_big_u64(), 25);
    is_all_zero(array_fn_should_use_mcli_big_u256(), 25);
    is_all_zero(array_fn_should_use_mcli_big_b256(), 25);
    is_all_zero(array_fn_should_use_mcli_big_bool(), 25);

    is_all_zero(array_fn_should_use_mcl_u8(), 262145);
}