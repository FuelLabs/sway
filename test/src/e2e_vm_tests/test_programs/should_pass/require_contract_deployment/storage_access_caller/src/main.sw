script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x3bc28acd66d327b8c1b9624c1fabfc07e9ffa1b5d71c2832c3bfaaf8f4b805e9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x44ef9b4bbc0178cbdb7070eab3eb4f9de25d85a3015e3845d8cfe93ed938e661; // AUTO-CONTRACT-ID ../../test_contracts/storage_access_contract --release

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
