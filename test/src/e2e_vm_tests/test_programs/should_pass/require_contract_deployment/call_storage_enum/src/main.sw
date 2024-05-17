script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0436d54f976e2dee0d77c81abc0d32cc7be985d8e0c97eeba27acd1caffdcea1;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xe7f37e7d15a223986ab1bbb6baca013f1ae6250566c3b49f0e8238db9c0dfc1a;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
