script;

use std::{assert::assert, contract_id::ContractId, logging::log};

const ETH_ID0 = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
fn wrapper(b: b256) -> ContractId {
    ~ContractId::from(b)
}
const ETH_ID1 = wrapper(0x0000000000000000000000000000000000000000000000000000000000000001);

const TUP1 = (2, 1, 21);
const ARR1 = [1, 2, 3];

fn main() -> u64 {
    // initialization through function applications.
    let eth_id0 = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000000);
    let eth_id1 = ~ContractId::from(0x0000000000000000000000000000000000000000000000000000000000000001);
    assert(eth_id0 == ETH_ID0 && eth_id1 == ETH_ID1);

    // tuples and arrays.
    let t1 = (2, 1, 21);
    assert(t1.0 == TUP1.0 && t1.1 == TUP1.1 && t1.2 == TUP1.2);
    let a1 = [1, 2, 3];
    assert(a1[0] == ARR1[0] && a1[1] == ARR1[1] && a1[2] == ARR1[2]);
    1
}
