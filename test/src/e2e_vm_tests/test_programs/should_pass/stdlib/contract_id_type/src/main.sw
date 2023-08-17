script;

use std::constants::{BASE_ASSET_ID, ZERO_B256};
use std::contract_id::*;

fn main() -> bool {
    let id_1 = ContractId::from(ZERO_B256);
    let id_2 = ContractId::from(ZERO_B256);

    assert(id_1 == id_2);
    assert(ZERO_B256 == id_1.into());

    true
}
