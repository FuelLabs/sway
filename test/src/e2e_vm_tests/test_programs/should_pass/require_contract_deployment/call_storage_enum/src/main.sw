script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0436d54f976e2dee0d77c81abc0d32cc7be985d8e0c97eeba27acd1caffdcea1;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x028e88f07ba7bf51d0b5d2e3cda7ccea35f8bdf788defa74ddb34841f074fcfd;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
