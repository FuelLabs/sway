script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x10ac00a805f5d051274a250b79047439e9df9c6c5626e0b4cecddc93e45e6ca3;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xc1d94591f6ea13e0a666939f12afab7a5db992559328032a8029c2c048d56632;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
