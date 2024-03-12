script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x10ac00a805f5d051274a250b79047439e9df9c6c5626e0b4cecddc93e45e6ca3;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x466d025c2cb81661c48d6dc045e01a59aab6f6d416943b08c8bde2d242e0b232;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
