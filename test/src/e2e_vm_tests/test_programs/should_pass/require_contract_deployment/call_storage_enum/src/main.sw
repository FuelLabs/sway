script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x10ac00a805f5d051274a250b79047439e9df9c6c5626e0b4cecddc93e45e6ca3;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x6420e2805bf99a7b8cb5a4a53fcf45fd208901c971dabbf56f7adc49f49a4d78;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
