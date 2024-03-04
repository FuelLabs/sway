script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xed4bbc1286211512f6894d6eded69eb27a8eaf551de44f10d2efb93088d9db82;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xed4bbc1286211512f6894d6eded69eb27a8eaf551de44f10d2efb93088d9db82;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
