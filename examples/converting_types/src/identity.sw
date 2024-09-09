library;

pub fn convert_to_identity() {
    let b256_address: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    // ANCHOR: convert_b256_to_address_or_contract_id
    let address_from_b256: Address = Address::from(b256_address);
    let contract_id_from_b256: ContractId = ContractId::from(b256_address);
    // ANCHOR_END: convert_b256_to_address_or_contract_id
    let address = address_from_b256;
    let contract_id = contract_id_from_b256;

    // ANCHOR: convert_to_identity
    let identity_from_b256: Identity = Identity::Address(Address::from(b256_address));
    let identity_from_address: Identity = Identity::Address(address);
    let identity_from_contract_id: Identity = Identity::ContractId(contract_id);
    // ANCHOR_END: convert_to_identity
}

pub fn convert_from_identity(my_identity: Identity) {
    // ANCHOR: convert_from_identity
    match my_identity {
        Identity::Address(address) => log(address),
        Identity::ContractId(contract_id) => log(contract_id),
    };
    // ANCHOR_END: convert_from_identity
}

pub fn convert_to_b256() {
    let b256_address: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let address: Address = Address::from(b256_address);
    let contract_id: ContractId = ContractId::from(b256_address);

    // ANCHOR: convert_to_b256
    let b256_from_address: b256 = address.into();
    let b256_from_contract_id: b256 = contract_id.into();
    // ANCHOR_END: convert_to_b256
}
