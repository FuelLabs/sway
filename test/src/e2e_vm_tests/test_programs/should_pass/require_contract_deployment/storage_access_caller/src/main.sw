script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xb0eee35cb9c3e2da8b5be0435192ea915d0e0dba2876528424af7bbb31574648;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xca7907061a4b00cf1ac85784ed617da0477c9c614dae74cd5c1ccba9f6720492;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
