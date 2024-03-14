script;

use storage_access_abi::*;
use std::hash::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xcd976bf8d7f3a9b54416c215ee0c732cbae4f9221e281fbc6c6aa8f428f03eb1;
#[cfg(experimental_new_encoding = true)]
<<<<<<< HEAD
const CONTRACT_ID = 0x61b5d0e4281ccc560bc434c86cb45460762c7d68c97c2e6b4d289c900f553acc;
=======
const CONTRACT_ID = 0x8eec91ef41980fe48d95624d31ecb116553af43073d434a8d93b261389e28151;
>>>>>>> 5a1a9d79c (updating contract ids)

fn main() -> bool {
    let caller = abi(StorageAccess, CONTRACT_ID);
    caller.set_boolean(true);
    assert(caller.get_boolean() == true);
    true
}
