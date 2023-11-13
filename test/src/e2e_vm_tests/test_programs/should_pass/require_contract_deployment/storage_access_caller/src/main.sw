script;

use storage_access_abi::*;
use std::hash::*;

fn main() -> bool {
    let contract_id = 0x8adff33262e366213a0479b44e0c24ce9e472275acb3077baf6a7ee58fe1cacc;
    let caller = abi(StorageAccess, contract_id);

    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    

    

    true
}
