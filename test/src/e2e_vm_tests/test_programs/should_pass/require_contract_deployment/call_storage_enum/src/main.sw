script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0436d54f976e2dee0d77c81abc0d32cc7be985d8e0c97eeba27acd1caffdcea1;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x3233079129b69c2e667a1b8bf5a0f81c52f5c379953a69ef2c851943be3c1ffc;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
