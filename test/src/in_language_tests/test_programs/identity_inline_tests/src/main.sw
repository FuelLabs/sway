library;

use std::hash::{Hash, sha256};

#[test]
fn identity_eq() {
    let address1 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    let address2 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    let address3 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let address4 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let address5 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let address6 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let contract_id1 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    let contract_id2 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    let contract_id3 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let contract_id4 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let contract_id5 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let contract_id6 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));

    assert(address1 == address2);
    assert(contract_id1 == contract_id2);
    assert(address3 == address4);
    assert(contract_id3 == contract_id4);
    assert(address5 == address6);
    assert(contract_id5 == contract_id6);
}

#[test]
fn identity_ne() {
    let address1 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    let address2 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    let address3 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let address4 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let address5 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let address6 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let contract_id1 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    let contract_id2 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000));
    let contract_id3 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let contract_id4 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let contract_id5 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let contract_id6 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));

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

    assert(contract_id1 != contract_id3);
    assert(contract_id1 != contract_id4);
    assert(contract_id1 != contract_id5);
    assert(contract_id1 != contract_id6);
    assert(contract_id2 != contract_id3);
    assert(contract_id2 != contract_id4);
    assert(contract_id2 != contract_id5);
    assert(contract_id2 != contract_id6);
    assert(contract_id3 != contract_id5);
    assert(contract_id3 != contract_id6);
    assert(contract_id4 != contract_id5);
    assert(contract_id4 != contract_id6);

    assert(address1 != contract_id1);
    assert(address1 != contract_id2);
    assert(address1 != contract_id3);
    assert(address1 != contract_id4);
    assert(address1 != contract_id5);
    assert(address1 != contract_id6);
    assert(address2 != contract_id1);
    assert(address2 != contract_id2);
    assert(address2 != contract_id3);
    assert(address2 != contract_id4);
    assert(address2 != contract_id5);
    assert(address2 != contract_id6);
    assert(address3 != contract_id1);
    assert(address3 != contract_id2);
    assert(address3 != contract_id3);
    assert(address3 != contract_id4);
    assert(address3 != contract_id5);
    assert(address3 != contract_id6);
    assert(address4 != contract_id1);
    assert(address4 != contract_id2);
    assert(address4 != contract_id3);
    assert(address4 != contract_id4);
    assert(address4 != contract_id5);
    assert(address4 != contract_id6);
    assert(address5 != contract_id1);
    assert(address5 != contract_id2);
    assert(address5 != contract_id3);
    assert(address5 != contract_id4);
    assert(address5 != contract_id5);
    assert(address5 != contract_id6);
    assert(address6 != contract_id1);
    assert(address6 != contract_id2);
    assert(address6 != contract_id3);
    assert(address6 != contract_id4);
    assert(address6 != contract_id5);
    assert(address6 != contract_id6);
}

#[test]
fn identity_as_address() {
    let address1 = Identity::Address(Address::zero());
    let address2 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let address3 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let contract_id1 = Identity::ContractId(ContractId::zero());
    let contract_id2 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let contract_id3 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));

    assert(address1.as_address().unwrap() == Address::zero());
    assert(contract_id1.as_address().is_none());
    assert(
        address2
            .as_address()
            .unwrap() == Address::from(0x0000000000000000000000000000000000000000000000000000000000000001),
    );
    assert(contract_id2.as_address().is_none());
    assert(
        address3
            .as_address()
            .unwrap() == Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF),
    );
    assert(contract_id3.as_address().is_none());
}

#[test]
fn identity_as_contract_id() {
    let address1 = Identity::Address(Address::zero());
    let address2 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let address3 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let contract_id1 = Identity::ContractId(ContractId::zero());
    let contract_id2 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let contract_id3 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));

    assert(address1.as_contract_id().is_none());
    assert(contract_id1.as_contract_id().unwrap() == ContractId::zero());
    assert(address2.as_contract_id().is_none());
    assert(
        contract_id2
            .as_contract_id()
            .unwrap() == ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001),
    );
    assert(address3.as_contract_id().is_none());
    assert(
        contract_id3
            .as_contract_id()
            .unwrap() == ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF),
    );
}

#[test]
fn identity_is_address() {
    let address1 = Identity::Address(Address::zero());
    let address2 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let address3 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let contract_id1 = Identity::ContractId(ContractId::zero());
    let contract_id2 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let contract_id3 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));

    assert(address1.is_address());
    assert(!contract_id1.is_address());
    assert(address2.is_address());
    assert(!contract_id2.is_address());
    assert(address3.is_address());
    assert(!contract_id3.is_address());
}

#[test]
fn identity_is_contract_id() {
    let address1 = Identity::Address(Address::zero());
    let address2 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let address3 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let contract_id1 = Identity::ContractId(ContractId::zero());
    let contract_id2 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let contract_id3 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));

    assert(!address1.is_contract_id());
    assert(contract_id1.is_contract_id());
    assert(!address2.is_contract_id());
    assert(contract_id2.is_contract_id());
    assert(!address3.is_contract_id());
    assert(contract_id3.is_contract_id());
}

#[test]
fn identity_bits() {
    let address1 = Identity::Address(Address::zero());
    let address2 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let address3 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
    let contract_id1 = Identity::ContractId(ContractId::zero());
    let contract_id2 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
    let contract_id3 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));

    assert(address1.bits() == b256::zero());
    assert(contract_id1.bits() == b256::zero());
    assert(
        address2
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        contract_id2
            .bits() == 0x0000000000000000000000000000000000000000000000000000000000000001,
    );
    assert(
        address3
            .bits() == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
    assert(
        contract_id3
            .bits() == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF,
    );
}

// #[test]
// fn identity_hash() {
//     let address1 = Identity::Address(Address::zero());
//     let address2 = Identity::Address(Address::from(0x0000000000000000000000000000000000000000000000000000000000000001));
//     let address3 = Identity::Address(Address::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));
//     let contract_id1 = Identity::ContractId(ContractId::zero());
//     let contract_id2 = Identity::ContractId(ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001));
//     let contract_id3 = Identity::ContractId(ContractId::from(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF));


//     assert(sha256(address1) == 0x0000000000000000000000000000000000000000000000000000000000000001);
//     assert(sha256(contract_id1) == b256::zero());
//     assert(sha256(address2) == 0x0000000000000000000000000000000000000000000000000000000000000001);
//     assert(sha256(contract_id2) == 0x0000000000000000000000000000000000000000000000000000000000000001);
//     assert(sha256(address3) == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
//     assert(sha256(contract_id3) == 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF);
// }
