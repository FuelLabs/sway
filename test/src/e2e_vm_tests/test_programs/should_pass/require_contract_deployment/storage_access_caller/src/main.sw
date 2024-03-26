script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xb0eee35cb9c3e2da8b5be0435192ea915d0e0dba2876528424af7bbb31574648;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x69100d93fdb5870d073083e86f8b8705f584d14956c3c88a5c1697a962437d1d;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
