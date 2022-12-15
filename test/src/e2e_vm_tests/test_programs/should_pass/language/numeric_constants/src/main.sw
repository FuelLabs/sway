script;

use std::assert::assert;

fn main() -> bool {
    assert(u64::max() == 18446744073709551615);
    assert(u64::min() == 0u64);
    assert(u64::bits() == 64u32);
    assert(u32::max() == 4294967295u32);
    assert(u32::min() == 0u32);
    assert(u32::bits() == 32u16);
    assert(u16::max() == 65535u16);
    assert(u16::min() == 0u16);
    assert(u16::bits() == 16u8);
    assert(u8::max() == 255u8);
    assert(u8::min() == 0u8);
    assert(u8::bits() == 8u32);

    true
}
