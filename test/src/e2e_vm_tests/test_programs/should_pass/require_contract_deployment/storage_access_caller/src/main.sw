script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0a58692bee60559887f0ac181c8a3b14ffb7a3a66256eec3f08e3135bfbecac9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x425da9054d1529627e50520ac245def2aa3e8a6a352c060b0bf95bad6c65dc11;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
