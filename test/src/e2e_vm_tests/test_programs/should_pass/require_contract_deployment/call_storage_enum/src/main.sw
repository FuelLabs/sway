script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0d2d9546e833c166b64a340f5694fa01ca6bb53c3ec681d6c1ade1b9c0a2bf46;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x92ee0b5ed85ac45c62ff340dbf75114329721ce101cb3dca1a26abcff0694826;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
