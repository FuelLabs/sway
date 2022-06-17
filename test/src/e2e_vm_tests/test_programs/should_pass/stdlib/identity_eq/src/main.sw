script;

use std::{address::Address, assert::assert, contract_id::ContractId, identity::Identity};

fn main() -> bool {
    let b1 = 0x0000000000000000000000000000000000000000000000000000000000000001;
    let b2 = 0x0000000000000000000000000000000000000000000000000000000000000002;

    let address1 = Identity::Address(~Address::from(b1));
    let address2 = Identity::Address(~Address::from(b2));
    let contract1 = Identity::ContractId(~ContractId::from(b1));
    let contract2 = Identity::ContractId(~ContractId::from(b2));

    // Eq is True
    assert(address1 == address1);
    assert(contract1 == contract1);

    // Eq is False
    assert(!(address1 == address2));
    assert(!(address2 == address1));

    assert(!(contract1 == contract2));
    assert(!(contract2 == contract1));

    assert(!(address1 == contract1));
    assert(!(contract1 == address1));

    // Neq is True
    assert(address1 != address2);
    assert(address2 != address1);

    assert(contract1 != contract2);
    assert(contract2 != contract1);

    assert(address1 != contract1);
    assert(contract1 != address1);

    // Neq is False
    assert(!(address1 != address1));
    assert(!(contract1 != contract1));

    true
}
