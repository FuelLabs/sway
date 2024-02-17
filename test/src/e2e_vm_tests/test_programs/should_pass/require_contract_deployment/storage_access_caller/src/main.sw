script;

use storage_access_abi::*;
use std::hash::*;

fn main() -> bool {
    let contract_id = 0xcd976bf8d7f3a9b54416c215ee0c732cbae4f9221e281fbc6c6aa8f428f03eb1;
    let caller = abi(StorageAccess, contract_id);

    caller.set_boolean(true);
    assert(caller.get_boolean() == true);

    true
}
