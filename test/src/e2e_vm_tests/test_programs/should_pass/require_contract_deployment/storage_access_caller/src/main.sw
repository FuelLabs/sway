script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x060a673b757bf47ce307548322586ec68b94a11ef330da149a7000435e3a294b;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xb01e2b0b2baecaf6751cf6a5ad731eee3ff5f6dd5f8ad3bcb85ea956d2b17f50;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
