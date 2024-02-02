script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x1cc4ed1d23bc6f78be934a6e019bab9346e75f79179ecbcf369544f2592b8031;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
