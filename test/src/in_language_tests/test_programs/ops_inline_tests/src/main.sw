library;

#[test]
pub fn str_eq_test() {
    assert("" == "");
    assert("a" == "a");

    assert("a" != "");
    assert("" != "a");
    assert("a" != "b");
}

#[test]
pub fn wrapping_tests() {
    assert_eq(0u8.wrapping_sub(1u8), u8::max());
    assert_eq(0u16.wrapping_sub(1u16), u16::max());
    assert_eq(0u32.wrapping_sub(1u32), u32::max());
    assert_eq(0u64.wrapping_sub(1u64), u64::max());
    assert_eq(u256::zero().wrapping_sub(u256::from(1u64)), u256::max());

    assert_eq(u8::max().wrapping_add(1u8), 0);
    assert_eq(u16::max().wrapping_add(1u16), 0);
    assert_eq(u32::max().wrapping_add(1u32), 0);
    assert_eq(u64::max().wrapping_add(1u64), 0);
    assert_eq(u256::max().wrapping_add(u256::from(1u64)), u256::zero());

    assert_eq(16u8.wrapping_mul(16u8), 0);
    assert_eq(256u16.wrapping_mul(256u16), 0);
    assert_eq(65_536u32.wrapping_mul(65_536u32), 0);
    assert_eq(4_294_967_296u64.wrapping_mul(4_294_967_296u64), 0);
    assert_eq(u256::from(0x0000000000000000000000000000000100000000000000000000000000000000).wrapping_mul(u256::from(0x0000000000000000000000000000000100000000000000000000000000000000)), 0);

}
