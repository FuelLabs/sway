script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xcd976bf8d7f3a9b54416c215ee0c732cbae4f9221e281fbc6c6aa8f428f03eb1;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x0f7b42a8c22f0355906aa3538fd6d46facc182c1daccd4a3c67537ac12576d7f;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
