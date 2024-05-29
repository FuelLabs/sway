script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x88732a14508defea37a44d0b0ae9af5c776253215180a1c3288f8d504ebb84db;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x257a60b6d43000783e2fff62eaccd9a4601484a44897f11b899cd12bbdde09d3;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
