script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x3bc28acd66d327b8c1b9624c1fabfc07e9ffa1b5d71c2832c3bfaaf8f4b805e9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x69648eda4704720d4c5c73ec9f5d63470e781cd549692987a88e4889e434f0f5; // AUTO-CONTRACT-ID ../../test_contracts/storage_access_contract --release

#[inline(never)]
fn call_contract_set_boolean() {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
}

fn main() -> bool {
    call_contract_set_boolean();

    let caller = abi(StorageAccess, CONTRACT_ID);
    assert(caller.get_boolean() == true);

    true
}
