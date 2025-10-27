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

#[test]
fn evm_address_try_from_bytes() {
    use std::bytes::Bytes;

    // Test empty bytes
    let bytes_1 = Bytes::new();
    assert(EvmAddress::try_from(bytes_1).is_none());

    // Test not full length but capacity bytes
    let mut bytes_2 = Bytes::with_capacity(20);
    bytes_2.push(1u8);
    bytes_2.push(3u8);
    bytes_2.push(5u8);
    assert(EvmAddress::try_from(bytes_2).is_none());

    // Test zero bytes
    let bytes_3_full = Bytes::from(b256::zero());
    let (bytes_3, _bytes_3_discard) = bytes_3_full.split_at(20);
    let evm_address_3 = EvmAddress::try_from(bytes_3);
    assert(evm_address_3.is_some());
    assert(evm_address_3.unwrap() == EvmAddress::zero());

    // Test max bytes
    let bytes_4_full = Bytes::from(b256::max());
    let (bytes_4, _bytes_4_discard) = bytes_4_full.split_at(20);
    let evm_address_4 = EvmAddress::try_from(bytes_4);
    assert(evm_address_4.is_some());
    assert(evm_address_4.unwrap() == EvmAddress::from(b256::max()));

    // Test too many bytes
    let bytes_5_full = Bytes::from(b256::zero());
    let (mut bytes_5, _bytes_5_discard) = bytes_5_full.split_at(20);
    bytes_5.push(255u8);
    assert(EvmAddress::try_from(bytes_5).is_none());

    // Test modifying bytes after doesn't impact 
    let bytes_6_full = Bytes::from(b256::zero());
    let (mut bytes_6, _bytes_6_discard) = bytes_6_full.split_at(20);
    let evm_address_6 = EvmAddress::try_from(bytes_6);
    assert(evm_address_6.is_some());
    assert(evm_address_6.unwrap() == EvmAddress::zero());
    bytes_6.set(0, 255u8);
    assert(evm_address_6.unwrap() == EvmAddress::zero());
}

#[test]
fn evm_address_into_bytes() {
    use std::bytes::Bytes;

    let evm_address_1 = EvmAddress::zero();
    let bytes_1: Bytes = <EvmAddress as Into<Bytes>>::into(evm_address_1);
    assert(bytes_1.capacity() == 20);
    assert(bytes_1.len() == 20);
    let mut iter_1 = 0;
    while iter_1 < 20 {
        assert(bytes_1.get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let evm_address_2 = EvmAddress::from(b256::max());
    let bytes_2: Bytes = <EvmAddress as Into<Bytes>>::into(evm_address_2);
    assert(bytes_2.capacity() == 20);
    assert(bytes_2.len() == 20);
    let mut iter_2 = 0;
    while iter_2 < 20 {
        assert(bytes_2.get(iter_2).unwrap() == 255u8);
        iter_2 += 1;
    }

    let evm_address_3 = EvmAddress::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let bytes_3: Bytes = <EvmAddress as Into<Bytes>>::into(evm_address_3);
    assert(bytes_3.capacity() == 20);
    assert(bytes_3.len() == 20);
    assert(bytes_3.get(19).unwrap() == 1u8);
    let mut iter_3 = 0;
    while iter_3 < 19 {
        assert(bytes_3.get(iter_3).unwrap() == 0u8);
        iter_3 += 1;
    }
}
