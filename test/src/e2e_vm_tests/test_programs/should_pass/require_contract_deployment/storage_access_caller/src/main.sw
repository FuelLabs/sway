script;

use storage_access_abi::*;
use std::hash::*;

fn main() -> bool {
    let contract_id = 0xf958d5fb187fd025750ecd9e99fd4a08d1816c21ac7e91d4414d93810f0e3383;
    let caller = abi(StorageAccess, contract_id);

    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    

    

    true
}
