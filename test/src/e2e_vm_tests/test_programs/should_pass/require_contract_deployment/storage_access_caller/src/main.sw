script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0a58692bee60559887f0ac181c8a3b14ffb7a3a66256eec3f08e3135bfbecac9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x9b350347974b8191648e05050abe8d8a94da21cbd7c984177bd55aad97c8edb5;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
