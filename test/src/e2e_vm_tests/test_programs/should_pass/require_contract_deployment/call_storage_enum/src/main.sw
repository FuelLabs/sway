script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x097ea9a771b69bf349375a0d6db542e9c730194de4bcd27e4e6665ffb107dfaf;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
