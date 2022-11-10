script;

dep test_lib;

use std::{assert::assert, contract_id::ContractId};

fn main() -> u64 {
    let x = test_lib::NUMBER;
    let zero = std::constants::ZERO_B256;
    let base_asset_id = std::constants::BASE_ASSET_ID;
    assert(zero == 0x0000000000000000000000000000000000000000000000000000000000000000);
    assert(base_asset_id == ContractId::from(zero));
    x
}
