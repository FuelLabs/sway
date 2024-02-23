script;

use storage_access_abi::*;
use std::hash::*;

fn main() -> bool {
    let contract_id = 0xed4bbc1286211512f6894d6eded69eb27a8eaf551de44f10d2efb93088d9db82;
    let caller = abi(StorageAccess, contract_id);

    caller.set_boolean(true);
    assert(caller.get_boolean() == true);

    true
}
