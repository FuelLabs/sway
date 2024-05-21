script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0a58692bee60559887f0ac181c8a3b14ffb7a3a66256eec3f08e3135bfbecac9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x8ae216e6df492a4a983c34c5871bbe84c6aafa70fa25f1d76b771d8c8a72a1d7;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
