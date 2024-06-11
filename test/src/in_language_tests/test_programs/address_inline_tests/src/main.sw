library;

#[test]
fn address_bits() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let bits1 = address1.bits();
    assert(bits1 == 0x0000000000000000000000000000000000000000000000000000000000000000);

    let address2 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let bits2 = address2.bits();
    assert(bits2 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let bits3 = address3.bits();
    assert(bits3 == 0x0000000000000000000000000000000000000000000000000000000000000001);
}

#[test]
fn address_eq() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let address2 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let address4 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let address5 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let address6 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    assert(address1 == address2);
    assert(address3 == address4);
    assert(address5 == address6);
}

#[test]
fn address_ne() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let address2 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let address4 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let address5 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let address6 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    assert(address1 != address3);
    assert(address1 != address4);
    assert(address1 != address5);
    assert(address1 != address6);
    assert(address2 != address3);
    assert(address2 != address4);
    assert(address2 != address5);
    assert(address2 != address6);
    assert(address3 != address5);
    assert(address3 != address6);
    assert(address4 != address5);
    assert(address4 != address6);
}

#[test]
fn address_from_b256() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(
        address1
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let address1 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    assert(
        address1
            .bits() == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(
        address3
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn address_b256_into() {
    let b256_1 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let address1: Address = b256_1.into();
    assert(
        address1
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let b256_2 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let address2: Address = b256_2.into();
    assert(
        address2
            .bits() == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let b256_3 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let address3: Address = b256_3.into();
    assert(
        address3
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn address_into_b256() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let b256_data1: b256 = address1.into();
    assert(
        b256_data1 == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let address2 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let b256_data2: b256 = address2.into();
    assert(
        b256_data2 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_data3: b256 = address3.into();
    assert(
        b256_data3 == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn address_b256_from() {
    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let b256_data1: b256 = b256::from(address1);
    assert(
        b256_data1 == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let address2 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let b256_data2: b256 = b256::from(address2);
    assert(
        b256_data2 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_data3: b256 = b256::from(address3);
    assert(
        b256_data3 == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn address_hash() {
    use std::hash::{Hash, sha256};

    let address1 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let digest1 = sha256(address1);
    assert(digest1 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let address2 = Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let digest2 = sha256(address2);
    assert(digest2 == 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);

    let address3 = Address::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let digest3 = sha256(address3);
    assert(digest3 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);

    let address4 = Address::from(0x4eaddc8cfcdd27223821e3e31ab54b2416dd3b0c1a86afd7e8d6538ca1bd0a77);
    let digest4 = sha256(address4);
    assert(digest4 != 0x4eaddc8cfcdd27223821e3e31ab54b2416dd3b0c1a86afd7e8d6538ca1bd0a77);
}

#[test]
fn address_zero() {
    let address = Address::zero();
    assert(
        address
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
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
