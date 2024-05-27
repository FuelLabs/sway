script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0d2d9546e833c166b64a340f5694fa01ca6bb53c3ec681d6c1ade1b9c0a2bf46;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x7688d1a9ea17b8d83a41e1ed8243e2c73dd8c3b9af1fb81ab95a0dd37268e3a4;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
