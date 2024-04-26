script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x060a673b757bf47ce307548322586ec68b94a11ef330da149a7000435e3a294b;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x8b2b4ceed11ec19b54ed8cf0c79ed71fd5c946990f4a849f417a0b9edc8bc91d;

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
