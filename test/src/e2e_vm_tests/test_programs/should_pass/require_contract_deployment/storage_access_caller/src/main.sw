script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0a58692bee60559887f0ac181c8a3b14ffb7a3a66256eec3f08e3135bfbecac9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x4afce106db8ef3bac5e2ec3c87043dd1df9bfd961a7bb4bfb997377ea5f4159e;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
