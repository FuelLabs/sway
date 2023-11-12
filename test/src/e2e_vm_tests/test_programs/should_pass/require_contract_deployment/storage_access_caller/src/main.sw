script;

use storage_access_abi::*;
use std::hash::*;

fn main() -> bool {
    let contract_id = 0x992a750d367980fc52fdfa9fa01481abd1cc6b12148cb70a4b0d2ba028b2d6dc;
    let caller = abi(StorageAccess, contract_id);

    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    

    

    true
}
