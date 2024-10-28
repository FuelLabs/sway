script;

fn main() -> bool {
    assert_eq(u64::max(), 18446744073709551615u64);
    assert_eq(u64::min(), 0u64);
    assert_eq(u32::max(), 4294967295u32);
    assert_eq(u32::min(), 0u32);
    assert_eq(u16::max(), 65535u16);
    assert_eq(u16::min(), 0u16);
    assert_eq(u8::max(), 255u8);
    assert_eq(u8::min(), 0u8);

    assert_eq(u64::bits(), 64u64);
    assert_eq(u32::bits(), 32u64);
    assert_eq(u16::bits(), 16u64);
    assert_eq(u8::bits(), 8u64);

    true
}
