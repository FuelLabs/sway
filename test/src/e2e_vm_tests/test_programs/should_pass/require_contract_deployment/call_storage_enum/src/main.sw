script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x0d2d9546e833c166b64a340f5694fa01ca6bb53c3ec681d6c1ade1b9c0a2bf46;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x62253b3aea7ea4986285f38db1f5202d1cf3c8a6684fd4f682370178095f84d0;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
