library b256;

use ::assert::assert;
use ::bytes::Bytes;
use ::convert::TryFrom;
use ::option::Option;

/**
 TODO: switch to match when we can do:
 match b {
    (b.len() > 32) => Option::None,
    _ => Option::Some(...)
}
*/
impl TryFrom<Bytes> for b256 {
    fn try_from(b: Bytes) -> Option<Self> {
        if b.len() > 32 {
            Option::None::<Self>
        } else {
            let mut val: b256 = 0x0000000000000000000000000000000000000000000000000000000000000000;
            let ptr = __addr_of(val);
            b.buf.ptr().copy_to::<b256>(ptr, 1);
            Option::Some(val)
        }
    }
}

#[test]
fn test_b256_try_from() {
    let mut initial_bytes = Bytes::with_capacity(32);
    // 0x33 is 51 in decimal
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);
    initial_bytes.push(51u8);

    let res = b256::try_from(initial_bytes);
    let expected = 0x3333333333333333333333333333333333333333333333333333333333333333;

    assert(res== expected);
}
