script;

use std::assert::assert;
use std::contract_id::ContractId;

fn main() -> bool {
    let bits = 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861;

    // test from()
    let id = ContractId::from(bits);
    assert(id.value == bits);

    // test into()
    let new_bits = id.into();
    assert(new_bits == bits);

    true
}
