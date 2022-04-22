script;

use std::constants::NATIVE_ASSET_ID;
use std::assert::assert;
use std::contract_id::*;

fn main() -> bool {
    let id_1 = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let id_2 = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);

    assert(id_1 == id_2);
    assert(id_1 == ~ContractId::from(NATIVE_ASSET_ID));
    assert(id_1.into() == NATIVE_ASSET_ID);

    true
}
