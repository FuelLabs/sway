script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x3bc28acd66d327b8c1b9624c1fabfc07e9ffa1b5d71c2832c3bfaaf8f4b805e9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xd13e88ba57bd2957c56ba97c462202fc88d98b5502e56d7cebaf02f5312bed06; // AUTO-CONTRACT-ID ../../test_contracts/storage_access_contract --release

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
