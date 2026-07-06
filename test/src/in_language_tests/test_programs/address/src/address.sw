library;

#[test]
fn address_bits() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let bits1 = address1.bits();
    assert_eq(
        bits1,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let address2 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let bits2 = address2.bits();
    assert_eq(
        bits2,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let bits3 = address3.bits();
    assert_eq(
        bits3,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn address_eq() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let address2 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let address4 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let address5 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let address6 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    assert_eq(address1, address2);
    assert_eq(address3, address4);
    assert_eq(address5, address6);
}

#[test]
fn address_ne() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let address2 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let address4 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let address5 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let address6 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    assert_ne(address1, address3);
    assert_ne(address1, address4);
    assert_ne(address1, address5);
    assert_ne(address1, address6);
    assert_ne(address2, address3);
    assert_ne(address2, address4);
    assert_ne(address2, address5);
    assert_ne(address2, address6);
    assert_ne(address3, address5);
    assert_ne(address3, address6);
    assert_ne(address4, address5);
    assert_ne(address4, address6);
}

#[test]
fn address_from_b256() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    assert_eq(
        address1
            .bits(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let address1 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    assert_eq(
        address1
            .bits(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert_eq(
        address3
            .bits(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn address_b256_into() {
    let b256_1 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let address1: Address = b256_1.into();
    assert_eq(
        address1
            .bits(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let b256_2 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let address2: Address = b256_2.into();
    assert_eq(
        address2
            .bits(),
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let b256_3 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let address3: Address = b256_3.into();
    assert_eq(
        address3
            .bits(),
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn address_into_b256() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let b256_data1: b256 = address1.into();
    assert_eq(
        b256_data1,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let address2 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let b256_data2: b256 = address2.into();
    assert_eq(
        b256_data2,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_data3: b256 = address3.into();
    assert_eq(
        b256_data3,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn address_b256_from() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let b256_data1: b256 = b256::from(address1);
    assert_eq(
        b256_data1,
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let address2 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let b256_data2: b256 = b256::from(address2);
    assert_eq(
        b256_data2,
        0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_data3: b256 = b256::from(address3);
    assert_eq(
        b256_data3,
        0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn address_hash() {
    use std::hash::{Hash, sha256};

    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let digest1 = sha256(address1);
    assert_eq(
        digest1,
        0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925,
    );

    let address2 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let digest2 = sha256(address2);
    assert_eq(
        digest2,
        0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051,
    );

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let digest3 = sha256(address3);
    assert_eq(
        digest3,
        0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5,
    );

    let address4 = Address::from(0x4eaddc8cfcdd27223821e3e31ab54b2416dd3b0c1a86afd7e8d6538ca1bd0a77);
    let digest4 = sha256(address4);
    assert_ne(
        digest4,
        0x4eaddc8cfcdd27223821e3e31ab54b2416dd3b0c1a86afd7e8d6538ca1bd0a77,
    );
}

#[test]
fn address_zero() {
    let address = Address::zero();
    assert_eq(
        address
            .bits(),
        0x0000000000000000000000000000000000000000000000000000000000000000,
    );
}

#[test]
fn address_is_zero() {
    let zero_address = Address::zero();
    assert(zero_address.is_zero());

    let address_2 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(!address_2.is_zero());

    let address_3 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    assert(!address_3.is_zero());
}

#[test]
fn address_try_from_bytes() {
    use std::bytes::Bytes;

    // Test empty bytes
    let bytes_1 = Bytes::new();
    assert(Address::try_from(bytes_1).is_none());

    // Test not full length but capacity bytes
    let mut bytes_2 = Bytes::with_capacity(32);
    bytes_2.push(1u8);
    bytes_2.push(3u8);
    bytes_2.push(5u8);
    assert(Address::try_from(bytes_2).is_none());

    // Test zero bytes
    let bytes_3 = Bytes::from(b256::zero());
    let address_3 = Address::try_from(bytes_3);
    assert(address_3.is_some());
    assert_eq(address_3.unwrap(), Address::zero());

    // Test max bytes
    let bytes_4 = Bytes::from(b256::max());
    let address_4 = Address::try_from(bytes_4);
    assert(address_4.is_some());
    assert_eq(address_4.unwrap(), Address::from(b256::max()));

    // Test too many bytes
    let mut bytes_5 = Bytes::from(b256::max());
    bytes_5.push(255u8);
    assert(Address::try_from(bytes_5).is_none());

    // Test modifying bytes after doesn't impact 
    let mut bytes_6 = Bytes::from(b256::zero());
    let address_6 = Address::try_from(bytes_6);
    assert(address_6.is_some());
    assert_eq(address_6.unwrap(), Address::zero());
    bytes_6.set(0, 255u8);
    assert_eq(address_6.unwrap(), Address::zero());
}

#[test]
fn address_try_into_bytes() {
    use std::bytes::Bytes;

    let address_1 = Address::zero();
    let bytes_1: Bytes = <Address as Into<Bytes>>::into(address_1);
    assert_eq(bytes_1.capacity(), 32);
    assert_eq(bytes_1.len(), 32);
    let mut iter_1 = 0;
    while iter_1 < 32 {
        assert_eq(bytes_1.get(iter_1).unwrap(), 0u8);
        iter_1 += 1;
    }

    let address_2 = Address::from(b256::max());
    let bytes_2: Bytes = <Address as Into<Bytes>>::into(address_2);
    assert_eq(bytes_2.capacity(), 32);
    assert_eq(bytes_2.len(), 32);
    let mut iter_2 = 0;
    while iter_2 < 32 {
        assert_eq(bytes_2.get(iter_2).unwrap(), 255u8);
        iter_2 += 1;
    }

    let address_3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let bytes_3: Bytes = <Address as Into<Bytes>>::into(address_3);
    assert_eq(bytes_3.capacity(), 32);
    assert_eq(bytes_3.len(), 32);
    assert_eq(bytes_3.get(31).unwrap(), 1u8);
    let mut iter_3 = 0;
    while iter_3 < 31 {
        assert_eq(bytes_3.get(iter_3).unwrap(), 0u8);
        iter_3 += 1;
    }
}
