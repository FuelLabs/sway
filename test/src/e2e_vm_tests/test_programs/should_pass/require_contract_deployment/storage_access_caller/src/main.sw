script;

use storage_access_abi::*;
use std::hash::*;

fn main() -> bool {
    let contract_id = 0x3c84d1eeaabed728e38cac455b1a64585228e54f0dd8f7c19f3028c9496cda0b;
    let caller = abi(StorageAccess, contract_id);

    caller.set_boolean(true);
    assert(caller.get_boolean() == true);

    true
}
