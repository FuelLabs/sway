library;

#[test]
fn contract_id_bits() {
    let contract_id_1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let bits1 = contract_id_1.bits();
    assert(bits1 == 0x0000000000000000000000000000000000000000000000000000000000000000);

    let contract_id_2 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let bits2 = contract_id_2.bits();
    assert(bits2 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    let contract_id_3 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let bits3 = contract_id_3.bits();
    assert(bits3 == 0x0000000000000000000000000000000000000000000000000000000000000001);
}

#[test]
fn contract_id_eq() {
    let contract_id_1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let contract_id_2 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let contract_id_3 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let contract_id_4 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let contract_id_5 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let contract_id_6 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    assert(contract_id_1 == contract_id_2);
    assert(contract_id_3 == contract_id_4);
    assert(contract_id_5 == contract_id_6);
}

#[test]
fn contract_id_ne() {
    let contract_id_1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let contract_id_2 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let contract_id_3 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let contract_id_4 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let contract_id_5 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let contract_id_6 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);

    assert(contract_id_1 != contract_id_3);
    assert(contract_id_1 != contract_id_4);
    assert(contract_id_1 != contract_id_5);
    assert(contract_id_1 != contract_id_6);
    assert(contract_id_2 != contract_id_3);
    assert(contract_id_2 != contract_id_4);
    assert(contract_id_2 != contract_id_5);
    assert(contract_id_2 != contract_id_6);
    assert(contract_id_3 != contract_id_5);
    assert(contract_id_3 != contract_id_6);
    assert(contract_id_4 != contract_id_5);
    assert(contract_id_4 != contract_id_6);
}

#[test]
fn contract_id_from_b256() {
    let contract_id_1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(
        contract_id_1
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let contract_id_2 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    assert(
        contract_id_2
            .bits() == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let contract_id_3 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(
        contract_id_3
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn contract_id_b256_into() {
    let b256_1 = 0x0000000000000000000000000000000000000000000000000000000000000000;
    let contract_id_1: ContractId = b256_1.into();
    assert(
        contract_id_1
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let b256_2 = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
    let contract_id_2: ContractId = b256_2.into();
    assert(
        contract_id_2
            .bits() == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let b256_3 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let contract_id_3: ContractId = b256_3.into();
    assert(
        contract_id_3
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn contract_id_into_b256() {
    let contract_id_1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let b256_data1: b256 = contract_id_1.into();
    assert(
        b256_data1 == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let contract_id_2 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let b256_data2: b256 = contract_id_2.into();
    assert(
        b256_data2 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let contract_id_3 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_data3: b256 = contract_id_3.into();
    assert(
        b256_data3 == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn contract_id_b256_from() {
    let contract_id_1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let b256_data1: b256 = b256::from(contract_id_1);
    assert(
        b256_data1 == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );

    let contract_id_2 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let b256_data2: b256 = b256::from(contract_id_2);
    assert(
        b256_data2 == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );

    let contract_id_3 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_data3: b256 = b256::from(contract_id_3);
    assert(
        b256_data3 == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
}

#[test]
fn contract_id_hash() {
    use std::hash::{Hash, sha256};

    let contract_id_1 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let digest1 = sha256(contract_id_1);
    assert(digest1 == 0x66687aadf862bd776c8fc18b8e9f8e20089714856ee233b3902a591d0d5f2925);

    let contract_id_2 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    let digest2 = sha256(contract_id_2);
    assert(digest2 == 0xaf9613760f72635fbdb44a5a0a63c39f12af30f950a6ee5c971be188e89c4051);

    let contract_id_3 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let digest3 = sha256(contract_id_3);
    assert(digest3 == 0xec4916dd28fc4c10d78e287ca5d9cc51ee1ae73cbfde08c6b37324cbfaac8bc5);

    let contract_id_4 = ContractId::from(0x4eaddc8cfcdd27223821e3e31ab54b2416dd3b0c1a86afd7e8d6538ca1bd0a77);
    let digest4 = sha256(contract_id_4);
    assert(digest4 != 0x4eaddc8cfcdd27223821e3e31ab54b2416dd3b0c1a86afd7e8d6538ca1bd0a77);
}

#[test]
fn contract_id_zero() {
    let contract_id = ContractId::zero();
    assert(
        contract_id
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000000,
    );
}

#[test]
fn contract_id_is_zero() {
    let zero_contract_id = ContractId::zero();
    assert(zero_contract_id.is_zero());

    let contract_id_2 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(!contract_id_2.is_zero());

    let contract_id_3 = ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
    assert(!contract_id_3.is_zero());
}

#[test]
fn contract_id_try_from_bytes() {
    use std::bytes::Bytes;

    // Test empty bytes
    let bytes_1 = Bytes::new();
    assert(ContractId::try_from(bytes_1).is_none());

    // Test not full length but capacity bytes
    let mut bytes_2 = Bytes::with_capacity(32);
    bytes_2.push(1u8);
    bytes_2.push(3u8);
    bytes_2.push(5u8);
    assert(ContractId::try_from(bytes_2).is_none());

    // Test zero bytes
    let bytes_3 = Bytes::from(b256::zero());
    let contract_id_3 = ContractId::try_from(bytes_3);
    assert(contract_id_3.is_some());
    assert(contract_id_3.unwrap() == ContractId::zero());

    // Test max bytes
    let bytes_4 = Bytes::from(b256::max());
    let contract_id_4 = ContractId::try_from(bytes_4);
    assert(contract_id_4.is_some());
    assert(contract_id_4.unwrap() == ContractId::from(b256::max()));

    // Test too many bytes
    let mut bytes_5 = Bytes::from(b256::max());
    bytes_5.push(255u8);
    assert(ContractId::try_from(bytes_5).is_none());

    // Test modifying bytes after doesn't impact 
    let mut bytes_6 = Bytes::from(b256::zero());
    let contract_id_6 = ContractId::try_from(bytes_6);
    assert(contract_id_6.is_some());
    assert(contract_id_6.unwrap() == ContractId::zero());
    bytes_6.set(0, 255u8);
    assert(contract_id_6.unwrap() == ContractId::zero());
}

#[test]
fn contract_id_try_into_bytes() {
    use std::bytes::Bytes;

    let contract_id_1 = ContractId::zero();
    let bytes_1: Bytes = <ContractId as Into<Bytes>>::into(contract_id_1);
    assert(bytes_1.capacity() == 32);
    assert(bytes_1.len() == 32);
    let mut iter_1 = 0;
    while iter_1 < 32 {
        assert(bytes_1.get(iter_1).unwrap() == 0u8);
        iter_1 += 1;
    }

    let contract_id_2 = ContractId::from(b256::max());
    let bytes_2: Bytes = <ContractId as Into<Bytes>>::into(contract_id_2);
    assert(bytes_2.capacity() == 32);
    assert(bytes_2.len() == 32);
    let mut iter_2 = 0;
    while iter_2 < 32 {
        assert(bytes_2.get(iter_2).unwrap() == 255u8);
        iter_2 += 1;
    }

    let contract_id_3 = ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    let bytes_3: Bytes = <ContractId as Into<Bytes>>::into(contract_id_3);
    assert(bytes_3.capacity() == 32);
    assert(bytes_3.len() == 32);
    assert(bytes_3.get(31).unwrap() == 1u8);
    let mut iter_3 = 0;
    while iter_3 < 31 {
        assert(bytes_3.get(iter_3).unwrap() == 0u8);
        iter_3 += 1;
    }
}
