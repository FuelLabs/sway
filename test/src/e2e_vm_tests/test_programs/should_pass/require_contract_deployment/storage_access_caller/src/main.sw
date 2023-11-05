script;

use storage_access_abi::*;
use std::hash::*;

fn main() -> bool {
    let contract_id = 0xc66a37f80e00d455bf7456caafa936091a884a409aab7f8337042d3d77b6fa34;
    let caller = abi(StorageAccess, contract_id);

    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    

    

    true
}
