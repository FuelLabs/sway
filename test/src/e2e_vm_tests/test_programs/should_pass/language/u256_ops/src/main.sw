script;

use core::ops::*;

fn main() -> bool {
    assert(0u256 == 0u256);
    assert(0u256 != 1u256);
    assert(2u256 == (1u256 + 1u256));
    assert(2u256 == (3u256 - 1u256));
    assert(10u256 == (5u256 * 2u256));
    assert(2u256 == (10u256 / 5u256));
    assert(2u256 == (12u256 % 5u256));
    // Not - still do not support big literals

    assert(1u256 > 0u256);
    assert(1u256 >= 0u256);
    assert(1u256 >= 1u256);

    assert(1u256 < 2u256);
    assert(1u256 <= 2u256);
    assert(2u256 <= 2u256);

    assert(1u256 == (1u256 & 1u256));
    assert(0u256 == (1u256 & 2u256));

    assert(1u256 == (1u256 | 1u256));
    assert(3u256 == (1u256 | 2u256));

    assert(0u256 == (1u256 ^ 1u256));
    assert(3u256 == (1u256 ^ 2u256));

    assert(8u256 == (1u256 << 3));
    assert(2u256 == (16u256 >> 3));

    // Errors
    // add overflow
    // minus underflow
    // mul overflow
    // divide by zero
    true
}
