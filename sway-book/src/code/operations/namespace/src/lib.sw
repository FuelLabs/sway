library namespace;

// ANCHOR: address
pub struct Address {
    value: b256,
}
// ANCHOR_END: address
// ANCHOR: contract_id
pub struct ContractId {
    value: b256,
}
// ANCHOR_END: contract_id
// ANCHOR: identity
pub enum Identity {
    Address: Address,
    ContractId: ContractId,
}
// ANCHOR_END: identity
fn address_cast() {
    // ANCHOR: address_cast
    let b256_variable1 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let my_address = ~Address::from(b256_variable1);
    let b256_variable2 = my_address.into();
    // b256_variable1 == b256_variable2
    // ANCHOR_END: address_cast
}

fn contract_id_cast() {
    // ANCHOR: contract_id_cast
    let b256_variable1 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let my_contract_id = ~ContractId::from(b256_variable1);
    let b256_variable2 = my_contract_id.into();
    // b256_variable1 == b256_variable2
    // ANCHOR_END: contract_id_cast
}

fn identity_cast() {
    // ANCHOR: identity_cast
    let raw_address = 0xddec0e7e6a9a4a4e3e57d08d080d71a299c628a46bc609aab4627695679421ca;
    let my_address_identity = Identity::Address(~Address::from(raw_address));
    let my_contract_identity = Identity::ContractId(~ContractId::from(raw_address));
    // ANCHOR_END: identity_cast
}
