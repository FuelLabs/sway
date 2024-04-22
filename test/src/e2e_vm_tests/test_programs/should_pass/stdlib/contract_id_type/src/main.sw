script;

use std::constants::ZERO_B256;
use std::contract_id::*;

fn main() -> bool {
    let id_1 = ContractId::from(ZERO_B256);
    let id_2 = ContractId::from(ZERO_B256);
    let id_1_b256: b256 = id_1.into();
    
    assert(id_1 == id_2);
    assert(ZERO_B256 == id_1_b256);

    true
}
