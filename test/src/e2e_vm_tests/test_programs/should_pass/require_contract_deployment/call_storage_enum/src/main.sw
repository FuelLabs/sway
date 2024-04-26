script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x1ce765336fbb4c4558c7f5753fad01a8549521b03e82bc94048fa512750b9554;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xee8e809af7ce3de220e1b0468c7c062413ee855abce4223b7187a385cd93f5f8;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
