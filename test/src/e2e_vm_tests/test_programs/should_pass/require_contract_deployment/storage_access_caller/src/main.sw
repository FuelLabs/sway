script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xcd976bf8d7f3a9b54416c215ee0c732cbae4f9221e281fbc6c6aa8f428f03eb1;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x3c3c2e05ccf4448075f6008331dc11ccbd434bef6c866e1e4157bb218d2de4ff;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
