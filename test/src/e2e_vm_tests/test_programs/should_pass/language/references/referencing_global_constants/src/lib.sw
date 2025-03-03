library;

pub struct S1 {
   pub x: u64,
   pub y: u64,
}

pub const TEST: S1 = S1 { x: 101, y: 111 };
pub const TEST_U64: u64 = 202;
pub const TEST_BOOL: bool = true;
pub const TEST_ARRAY: [u64; 3] = [1, 2, 3];
pub const TEST_TUPLE: (u64, bool, [u64; 3]) = (303, false, [4, 5, 6]);

#[inline(never)]
pub fn get_addr_tuple() -> u64 {
   let tuple_addr = &TEST_TUPLE;
   reference_to_int(tuple_addr)
}

#[test]
fn test_tuple_addr() {
   assert(get_addr_tuple() == reference_to_int(&TEST_TUPLE));
}
#[inline(never)]
pub fn get_addr_u64() -> u64 {
   let u64_addr = &TEST_U64;
   reference_to_int(u64_addr)
}

#[inline(never)]
pub fn get_addr_bool() -> u64 {
   let bool_addr = &TEST_BOOL;
   reference_to_int(bool_addr)
}

#[inline(never)]
pub fn get_addr_array() -> u64 {
   let array_addr = &TEST_ARRAY;
   reference_to_int(array_addr)
}

#[test]
fn test_u64_addr() {
   assert(get_addr_u64() == reference_to_int(&TEST_U64));
}

#[test]
fn test_bool_addr() {
   assert(get_addr_bool() == reference_to_int(&TEST_BOOL));
}

#[test]
fn test_array_addr() {
   assert(get_addr_array() == reference_to_int(&TEST_ARRAY));
}
impl S1 {
   const ASSOCIATED_CONST: u64 = 123;
}

pub fn reference_to_int<T>(ptr: &T) -> u64 {
   asm(ptr:ptr) {
        ptr: u64
   }
}

#[inline(never)]
pub fn get_addr_x() -> u64 {
    let x_addr = &TEST.x;
    reference_to_int(x_addr)
}

#[inline(never)]
pub fn get_addr_y() -> u64 {
    let y_addr = &TEST.y;
    reference_to_int(y_addr)
}

#[inline(never)]
pub fn sum_x_y_addresses() -> u64 {
   get_addr_x() + get_addr_y()
}

#[inline(never)]
pub fn get_associated_const() -> u64 {
   let assoc_const = &S1::ASSOCIATED_CONST;
   reference_to_int(assoc_const)
}

#[test]
fn test_x_y_addr() {
   assert(get_addr_x() + get_addr_y() == sum_x_y_addresses());
}
