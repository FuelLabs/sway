script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0a58692bee60559887f0ac181c8a3b14ffb7a3a66256eec3f08e3135bfbecac9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x9cf289ac9b774d01a293a23956457e907a0b7edf0a84b4b9be83c873c6fb03c0;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
