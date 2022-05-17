script;

use std::intrinsics::*;
use std::assert::assert;

fn main() -> bool {
    assert(!is_reference_type(42u64));
    assert(!is_reference_type(42u32));
    assert(!is_reference_type(42u16));
    assert(!is_reference_type(11u8));
    assert(!is_reference_type(false));

    true
}
