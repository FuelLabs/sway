library;

use std::vm::evm::evm_address::EvmAddress;

#[test]
fn evm_address_bits() {
    let evm_address_1 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let evm_address_2 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let evm_address_3 = EvmAddress::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);

    assert(
        evm_address_1
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(
        evm_address_2
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        evm_address_3
            .bits() == 0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff,
    );
}

#[test]
fn evm_address_eq() {
    let evm_address_1 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let evm_address_2 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let evm_address_3 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let evm_address_4 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let evm_address_5 = EvmAddress::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);
    let evm_address_6 = EvmAddress::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);

    assert(evm_address_1 == evm_address_2);
    assert(evm_address_3 == evm_address_4);
    assert(evm_address_5 == evm_address_6);
}

#[test]
fn evm_address_ne() {
    let evm_address_1 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let evm_address_2 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let evm_address_3 = EvmAddress::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);

    assert(evm_address_1 != evm_address_2);
    assert(evm_address_1 != evm_address_3);
    assert(evm_address_2 != evm_address_3);
}

#[test]
fn evm_address_from_b256() {
    let evm_address_1 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let evm_address_2 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let evm_address_3 = EvmAddress::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);

    assert(
        evm_address_1
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
    assert(
        evm_address_2
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        evm_address_3
            .bits() == 0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff,
    );
}

#[test]
fn evm_address_into_b256() {
    let evm_address_1 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let evm_address_2 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let evm_address_3 = EvmAddress::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);

    let b256_1: b256 = evm_address_1.into();
    let b256_2: b256 = evm_address_2.into();
    let b256_3: b256 = evm_address_3.into();

    assert(b256_1 == 0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(b256_2 == 0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(b256_3 == 0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff);
}

#[test]
fn evm_address_b256_from() {
    // Glob operator is needed for from for b256
    use std::vm::evm::evm_address::*;

    let evm_address_1 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let evm_address_2 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let evm_address_3 = EvmAddress::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);

    let b256_1 = b256::from(evm_address_1);
    let b256_2 = b256::from(evm_address_2);
    let b256_3 = b256::from(evm_address_3);

    assert(b256_1 == 0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(b256_2 == 0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(b256_3 == 0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff);
}

#[test]
fn evm_address_b256_into() {
    // Glob operator is needed for into for b256
    use std::vm::evm::evm_address::*;

    let evm_address_1 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let evm_address_2 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let evm_address_3 = EvmAddress::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);

    let b256_1: b256 = evm_address_1.into();
    let b256_2: b256 = evm_address_2.into();
    let b256_3: b256 = evm_address_3.into();

    assert(b256_1 == 0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(b256_2 == 0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(b256_3 == 0x000000000000000000000000ffffffffffffffffffffffffffffffffffffffff);
}

#[test]
fn evm_address_zero() {
    let evm_address = EvmAddress::zero();
    assert(
        evm_address
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
}

#[test]
fn evm_address_is_zero() {
    let evm_address_1 = EvmAddress::zero();
    let evm_address_2 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let evm_address_3 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let evm_address_4 = EvmAddress::from(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff);

    assert(evm_address_1.is_zero());
    assert(evm_address_2.is_zero());
    assert(!evm_address_3.is_zero());
    assert(!evm_address_4.is_zero());
}

#[test]
fn evm_address_hash() {
    use std::hash::{Hash, sha256};

    let evm_address_1 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let digest_1 = sha256(evm_address_1);
    assert(digest_1 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let evm_address_2 = EvmAddress::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let digest_2 = sha256(evm_address_2);
    assert(digest_2 == 0x78230345cedf8e92525c3cfdb8a95e947de1ed72e881b055dd80f9e523ff33e0);

    let evm_address_3 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let digest_3 = sha256(evm_address_3);
    assert(digest_3 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);

    let evm_address_4 = EvmAddress::from(0x4eaddc8cfcdd27223821e3e31ab54b2416dd3b0c1a86afd7e8d6538ca1bd0a77);
    let digest_4 = sha256(evm_address_4);
    assert(digest_4 != 0x4eaddc8cfcdd27223821e3e31ab54b2416dd3b0c1a86afd7e8d6538ca1bd0a77);
}
