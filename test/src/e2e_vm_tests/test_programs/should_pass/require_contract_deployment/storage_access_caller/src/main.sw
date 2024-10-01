script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x3bc28acd66d327b8c1b9624c1fabfc07e9ffa1b5d71c2832c3bfaaf8f4b805e9;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
const CONTRACT_ID = 0xdc525dd2ea57aa691d4d61acd725ebbefd974eaeb397598ca90c38807dc749bb; // AUTO-CONTRACT-ID ../../test_contracts/storage_access_contract --release
=======
const CONTRACT_ID = 0x83ec366b3623ee28ec09d3d92dcc2e113cc7427adb194fb42608103b0188fb83; // AUTO-CONTRACT-ID ../../test_contracts/storage_access_contract --release
>>>>>>> bb1c14b4e (dont break backward compatability on conditional compilation)

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
