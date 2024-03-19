script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xb0eee35cb9c3e2da8b5be0435192ea915d0e0dba2876528424af7bbb31574648;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x17d89bbf1b18f6388dffcc61c9fc2e92122cf4d11d171ab746a7cac9800cc12c;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
