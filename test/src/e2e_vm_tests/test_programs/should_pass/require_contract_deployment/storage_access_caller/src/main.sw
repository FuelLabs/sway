script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x060a673b757bf47ce307548322586ec68b94a11ef330da149a7000435e3a294b;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xd258b6303f783a49a8c9370bc824b0824a06edf05a3299b8c9e69d6bf26d2149;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
