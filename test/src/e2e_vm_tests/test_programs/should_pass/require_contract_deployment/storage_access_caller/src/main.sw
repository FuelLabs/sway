script;

use storage_access_abi::*;
use std::hash::*;

fn main() -> bool {
    let contract_id = 0x9f807040099c184c7784fd61e1c9d200244d3e2130a486a32c6a20a90cf3616f;
    let caller = abi(StorageAccess, contract_id);

    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    

    
    

    

    true
}
