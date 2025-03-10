library;

use std::flags::{disable_panic_on_overflow, disable_panic_on_unsafe_math};

//u8
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
fn math_u8_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
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
fn math_u8_zero_root_disable_overflow() {
    let _ = disable_panic_on_overflow();
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

#[test(should_revert)]
fn math_u8_0th_root_fail() {
    let res = asm(r1: 100u8, r2: 0u8, r3) {
        mroo r3 r1 r2;
        r3: u8
    };
    log(res);
}

// u16
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
fn math_u16_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
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
fn math_u16_disable_overflow() {
    let _ = disable_panic_on_overflow();
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

#[test(should_revert)]
fn math_u16_0th_root_fail() {
    let res = asm(r1: 100u16, r2: 0u16, r3) {
        mroo r3 r1 r2;
        r3: u16
    };
    log(res);
}

// u32
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
fn math_u32_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
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
fn math_u32_disable_overflow() {
    let _ = disable_panic_on_overflow();
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

#[test(should_revert)]
fn math_u32_0th_root_fail() {
    let res = asm(r1: 100u32, r2: 0u32, r3) {
        mroo r3 r1 r2;
        r3: u32
    };
    log(res);
}

// u64
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
fn math_u64_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
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
fn math_u64_disable_overflow() {
    let _ = disable_panic_on_overflow();
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

#[test(should_revert)]
fn math_u64_0th_root_fail() {
    let res = asm(r1: 100u64, r2: 0u64, r3) {
        mroo r3 r1 r2;
        r3: u64
    };
    log(res);
}

// u256
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
fn math_root_u256_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();
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
fn math_root_u256_disable_overflow() {
    let _ = disable_panic_on_overflow();
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

// #[test(should_revert)]
// fn math_u256_0th_root_fail() {
//     let res = asm(r1: 0x64u256, r2: 0x00u256, r3) {
//         mroo r3 r1 r2;
//         r3: u256
//     };
//     log(res);
// }
