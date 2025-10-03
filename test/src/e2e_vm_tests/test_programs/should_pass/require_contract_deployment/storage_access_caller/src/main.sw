script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x3bc28acd66d327b8c1b9624c1fabfc07e9ffa1b5d71c2832c3bfaaf8f4b805e9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x373f6ff58b11734d1ca4276955c68295b4f1fcfcccbb5417f43a26cdea2f7dbf; // AUTO-CONTRACT-ID ../../test_contracts/storage_access_contract --release

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
