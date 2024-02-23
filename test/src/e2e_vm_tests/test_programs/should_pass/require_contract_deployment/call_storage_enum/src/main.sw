script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x4c01b41e6f7fc88c88a7799c43d9f695e22ee01eed90478b99fe3bfa935e3e07;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
