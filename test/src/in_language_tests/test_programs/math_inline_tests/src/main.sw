library;

#[test]
fn math_root_u256() {
    let max_u256 = u256::max();

    assert(0x1u256.sqrt() == 1);
    assert(0x4u256.sqrt() == 2);
    assert(0x9u256.sqrt() == 3);
    assert(0x90u256.sqrt() == 12);
    assert(0x400u256.sqrt() == 32);
    assert(0x2386f26fc10000u256.sqrt() == 100000000);
    assert(0x0u256.sqrt() == 0);
    assert(0x2u256.sqrt() == 1);
    assert(0x5u256.sqrt() == 2);
    assert(0x3e8u256.sqrt() == 31);
    assert(max_u256.sqrt() == 0xffffffffffffffffffffffffffffffffu256);
}

#[test]
fn math_root_u64() {
    let max_u64 = u64::max();

    assert(1.sqrt() == 1);
    assert(4.sqrt() == 2);
    assert(9.sqrt() == 3);
    assert(144.sqrt() == 12);
    assert(1024.sqrt() == 32);
    assert(10000000000000000.sqrt() == 100000000);
    assert(0.sqrt() == 0);
    assert(2.sqrt() == 1);
    assert(5.sqrt() == 2);
    assert(1000.sqrt() == 31);
    assert(max_u64.sqrt() == 4294967295);
}

#[test]
fn math_root_u32() {
    let max_u32 = u32::max();

    assert(1u32.sqrt() == 1);
    assert(4u32.sqrt() == 2);
    assert(9u32.sqrt() == 3);
    assert(144u32.sqrt() == 12);
    assert(1024u32.sqrt() == 32);
    assert(100000000u32.sqrt() == 10000);
    assert(0u32.sqrt() == 0);
    assert(2u32.sqrt() == 1);
    assert(5u32.sqrt() == 2);
    assert(1000u32.sqrt() == 31);
    assert(max_u32.sqrt() == 65535);
}

#[test]
fn math_root_u16() {
    let max_u16 = u16::max();

    assert(1u16.sqrt() == 1);
    assert(4u16.sqrt() == 2);
    assert(9u16.sqrt() == 3);
    assert(144u16.sqrt() == 12);
    assert(1024u16.sqrt() == 32);
    assert(50625u16.sqrt() == 225);
    assert(0u16.sqrt() == 0);
    assert(2u16.sqrt() == 1);
    assert(5u16.sqrt() == 2);
    assert(1000u16.sqrt() == 31);
    assert(max_u16.sqrt() == 255);
}

#[test]
fn math_root_u8() {
    let max_u8 = u8::max();

    assert(1u8.sqrt() == 1);
    assert(4u8.sqrt() == 2);
    assert(9u8.sqrt() == 3);
    assert(144u8.sqrt() == 12);
    assert(0u8.sqrt() == 0);
    assert(2u8.sqrt() == 1);
    assert(5u8.sqrt() == 2);
    assert(max_u8.sqrt() == 15);
}

#[test]
fn math_power_u256() {
    let five = 0x0000000000000000000000000000000000000000000000000000000000000005u256;

    // 5^2 = 25 = 0x19
    assert_eq(
        five
            .pow(2),
        0x0000000000000000000000000000000000000000000000000000000000000019u256,
    );

    // 5^28 = 0x204FCE5E3E2502611 (see https://www.wolframalpha.com/input?i=convert+5%5E28+in+hex)
    assert_eq(five.pow(28), 0x0000000000000000204FCE5E3E2502611u256);
}

#[test]
fn math_power_u64() {
    assert(2.pow(2) == 4);
    assert(2 ** 2 == 4);

    assert(2.pow(3) == 8);
    assert(2 ** 3 == 8);

    assert(42.pow(2) == 1764);
    assert(42 ** 2 == 1764);

    assert(42.pow(3) == 74088);
    assert(42 ** 3 == 74088);

    assert(100.pow(5) == 10000000000);
    assert(100 ** 5 == 10000000000);

    assert(100.pow(8) == 10000000000000000);
    assert(100 ** 8 == 10000000000000000);

    assert(100.pow(9) == 1000000000000000000);
    assert(100 ** 9 == 1000000000000000000);

    assert(2.pow(0) == 1);
    assert(2 ** 0 == 1);

    assert(0.pow(1) == 0);
    assert(0 ** 1 == 0);

    assert(0.pow(2) == 0);
    assert(0 ** 2 == 0);
}

#[test]
fn math_power_u32() {
    assert(2u32.pow(2u32) == 4u32);
    assert(2u32 ** 2u32 == 4u32);

    assert(2u32.pow(3u32) == 8u32);
    assert(2u32 ** 3u32 == 8u32);

    assert(42u32.pow(2u32) == 1764u32);
    assert(42u32 ** 2u32 == 1764u32);

    assert(100u32.pow(4u32) == 100000000u32);
    assert(100u32 ** 4u32 == 100000000u32);

    assert(2u32.pow(0u32) == 1u32);
    assert(2u32 ** 0u32 == 1u32);

    assert(0u32.pow(1u32) == 0u32);
    assert(0u32 ** 1u32 == 0u32);

    assert(0u32.pow(2u32) == 0u32);
    assert(0u32 ** 2u32 == 0u32);
}

#[test]
fn math_power_u16() {
    assert(2u16.pow(2u32) == 4u16);
    assert(2u16 ** 2u32 == 4u16);

    assert(2u16.pow(3u32) == 8u16);
    assert(2u16 ** 3u32 == 8u16);

    assert(42u16.pow(2u32) == 1764u16);
    assert(42u16 ** 2u32 == 1764u16);

    assert(20u16.pow(3u32) == 8000u16);
    assert(20u16 ** 3u32 == 8000u16);

    assert(15u16.pow(4u32) == 50625u16);
    assert(15u16 ** 4u32 == 50625u16);

    assert(2u16.pow(0u32) == 1u16);
    assert(2u16 ** 0u32 == 1u16);

    assert(0u16.pow(1u32) == 0u16);
    assert(0u16 ** 1u32 == 0u16);

    assert(0u16.pow(2u32) == 0u16);
    assert(0u16 ** 2u32 == 0u16);
}

#[test]
fn math_power_u8() {
    assert(2u8.pow(2u32) == 4u8);
    assert(2u8 ** 2u32 == 4u8);

    assert(2u8.pow(3u32) == 8u8);
    assert(2u8 ** 3u32 == 8u8);

    assert(4u8.pow(3u32) == 64u8);
    assert(4u8 ** 3u32 == 64u8);

    assert(3u8.pow(4u32) == 81u8);
    assert(3u8 ** 4u32 == 81u8);

    assert(10u8.pow(2u32) == 100u8);
    assert(10u8 ** 2u32 == 100u8);

    assert(5u8.pow(3u32) == 125u8);
    assert(5u8 ** 3u32 == 125u8);

    assert(3u8.pow(5u32) == 243u8);
    assert(3u8 ** 5u32 == 243u8);

    assert(2u8.pow(0u32) == 1u8);
    assert(2u8 ** 0u32 == 1u8);

    assert(0u8.pow(1u32) == 0u8);
    assert(0u8 ** 1u32 == 0u8);

    assert(0u8.pow(2u32) == 0u8);
    assert(0u8 ** 2u32 == 0u8);
}

#[test]
fn math_log_u64() {
    let max_u64 = u64::max();

    assert(2.log(2) == 1);
    assert(2.log2() == 1);
    assert(1.log(3) == 0);
    assert(8.log(2) == 3);
    assert(8.log2() == 3);
    assert(100.log(10) == 2);
    assert(100.log(2) == 6);
    assert(100.log2() == 6);
    assert(100.log(9) == 2);
    assert(max_u64.log(10) == 19);
    assert(max_u64.log(2) == 63);
}

#[test]
fn math_log_u32() {
    let max_u32 = u32::max();

    assert(2u32.log(2u32) == 1u32);
    assert(100u32.log(10u32) == 2u32);
    assert(125u32.log(5u32) == 3u32);
    assert(256u32.log(4u32) == 4u32);
    assert(max_u32.log(10) == 9);
    assert(max_u32.log(2) == 31);
}

#[test]
fn math_log_u16() {
    let max_u16 = u16::max();

    assert(7u16.log(7u16) == 1u16);
    assert(49u16.log(7u16) == 2u16);
    assert(27u16.log(3u16) == 3u16);
    assert(1024u16.log(2u16) == 10u16);
    assert(max_u16.log(10) == 4);
    assert(max_u16.log(2) == 15);
}

#[test]
fn math_log_u8() {
    let max_u8 = u8::max();

    assert(20u8.log(20u8) == 1u8);
    assert(81u8.log(9u8) == 2u8);
    assert(36u8.log(6u8) == 2u8);
    assert(125u8.log(5u8) == 3u8);
    assert(max_u8.log(10) == 2);
    assert(max_u8.log(2) == 7);
}

#[test]
fn math_log_u256() {
    let max_u256 = u256::max();
    assert(0x2u256.log(0x2u256) == 0x1u256);
    assert(0x1u256.log(0x3u256) == 0);
    assert(0x8u256.log(0x2u256) == 0x3u256);
    assert(0x64u256.log(0xau256) == 0x2u256);
    assert(0x64u256.log(0x2u256) == 0x6u256);
    assert(0x64u256.log(0x9u256) == 0x2u256);
    assert(max_u256.log(0x2u256) == 0xffu256);
}

#[test]
fn math_log2_u256() {
    let max_u256 = u256::max();
    assert(0x2u256.log2() == 0x1u256);
    assert(0x401u256.log2() == 0xau256);
    assert(max_u256.log2() == 0xffu256);
    assert(0x2u256.log2() == 0x1u256);
    assert(0x8u256.log2() == 0x3u256);
    assert(0x64u256.log2() == 0x6u256);
}

#[test]
fn math_log2_u64() {
    let max_u64 = u64::max();
    assert(max_u64.log2() == 63);
}

#[test]
fn math_log2_u32() {
    let max_u32 = u32::max();
    assert(max_u32.log2() == 31);
}

#[test]
fn math_log2_u16() {
    let max_u16 = u16::max();
    assert(max_u16.log2() == 15);
}

#[test]
fn math_log2_u8() {
    let max_u8 = u8::max();
    assert(max_u8.log2() == 7);
}

#[test]
fn math_u8_zero() {
    let my_u8 = u8::zero();
    assert(my_u8.is_zero());

    let other_u8 = 1u8;
    assert(!other_u8.is_zero());
}

#[test]
fn math_u16_zero() {
    let my_u16 = u16::zero();
    assert(my_u16.is_zero());

    let other_u16 = 1u16;
    assert(!other_u16.is_zero());
}

#[test]
fn math_u32_zero() {
    let my_u32 = u32::zero();
    assert(my_u32.is_zero());

    let other_u32 = 1u32;
    assert(!other_u32.is_zero());
}

#[test]
fn math_u64_zero() {
    let my_u64 = u64::zero();
    assert(my_u64.is_zero());

    let other_u64 = 1u64;
    assert(!other_u64.is_zero());
}

#[test]
fn math_u256_zero() {
    let my_u256 = u256::zero();
    assert(my_u256.is_zero());

    let other_u256 = 0x01u256;
    assert(!other_u256.is_zero());
}

#[test]
fn math_b256_zero() {
    let my_b256 = b256::zero();
    assert(my_b256.is_zero());

    let other_b256 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    assert(!other_b256.is_zero());
}
