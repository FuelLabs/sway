library;

use ::assert::assert;
use ::bytes::Bytes;
use ::convert::TryFrom;
use ::option::Option::{*, self};
use ::logging::log;
use ::math::*;

impl TryFrom<Bytes> for b256 {
    fn try_from(b: Bytes) -> Option<Self> {
        if b.len() > 32 {
            None
        } else {
            let mut val = 0x0000000000000000000000000000000000000000000000000000000000000000;
            let ptr = __addr_of(val);
            b.buf.ptr().copy_to::<b256>(ptr, 1);
            Some(val)
        }
    }
}

impl b256 {
    // Increments a b256 by a u64 amount
    fn increment(ref mut self, amount: u64) {
        // Decompose the b256 into 4 words
        let (mut word1, mut word2, mut word3, mut word4) = asm(r1: self) { r1: (u64, u64, u64, u64) };

        // Add the amount and carry the overflow to the next word
        let (overflow4, result4) = word4.overflowing_add(amount);
        let (overflow3, result3) = word3.overflowing_add(overflow4);
        let (overflow2, result2) = word2.overflowing_add(overflow3);
        let (overflow1, result1) = word1.overflowing_add(overflow2);
        // If word1 overflows then we have reached past the max of a b256
        assert(overflow1 == 0);

        // Recompose the b256 and assign to self
        self = asm(r1: (result1, result2, result3, result4)) { r1: b256 };
    }
}

#[test]
fn test_b256_try_from() {
    let mut initial_bytes = Bytes::with_capacity(32);
    let mut i = 0;
    while i < 32 {
        // 0x33 is 51 in decimal
        initial_bytes.push(51u8);
        i += 1;
    }
    let res = b256::try_from(initial_bytes);
    let expected = 0x3333333333333333333333333333333333333333333333333333333333333333;

    assert(res.unwrap() == expected);

    let mut second_bytes = Bytes::with_capacity(33);
    i = 0;
    while i < 33 {
        // 0x33 is 51 in decimal
        second_bytes.push(51u8);
        i += 1;
    }
    let res = b256::try_from(second_bytes);
    assert(res.is_none());

    // bytes is still available to use:
    assert(second_bytes.len() == 33);
    assert(second_bytes.capacity() == 33);
}

#[test]
fn test_b256_increment() {
    let mut val = 0x0000000000000000000000000000000000000000000000000000000000000000;

    val.increment(1);
    assert(val == 0x0000000000000000000000000000000000000000000000000000000000000001);

    val.increment(1);
    assert(val == 0x0000000000000000000000000000000000000000000000000000000000000002);

    val.increment(8);
    assert(val == 0x000000000000000000000000000000000000000000000000000000000000000a);

    // force overflow
    val.increment(u64::max());
    assert(val == 0x0000000000000000000000000000000000000000000000010000000000000009);

    val.increment(u64::max());
    assert(val == 0x0000000000000000000000000000000000000000000000020000000000000008);
}
