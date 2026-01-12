script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x3bc28acd66d327b8c1b9624c1fabfc07e9ffa1b5d71c2832c3bfaaf8f4b805e9;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
<<<<<<< HEAD
const CONTRACT_ID = 0x4438abd8ec205e2661f3cfb589e5d053e4d6ef645125c9632348d9aab0350c0d; // AUTO-CONTRACT-ID ../../test_contracts/storage_access_contract --release
=======
const CONTRACT_ID = 0x7e18ae0c2393b0b1bf2bdd3ea84cb3806c3cde9446bdf77fdda6e93a3c275446; // AUTO-CONTRACT-ID ../../test_contracts/storage_access_contract --release
>>>>>>> c3542add6 (update tests)
=======
const CONTRACT_ID = 0x42ea20a249b50824cc8f63a9cc3ec21966007aa8089f80d0dcffdb207307e96c; // AUTO-CONTRACT-ID ../../test_contracts/storage_access_contract --release
>>>>>>> 457ab385d (update tests)

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
