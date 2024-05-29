script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x88732a14508defea37a44d0b0ae9af5c776253215180a1c3288f8d504ebb84db;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x69a1c9d2ddba33626420fe322b230deb9b4f519b73ff787aab0a824a2290d1ee;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
