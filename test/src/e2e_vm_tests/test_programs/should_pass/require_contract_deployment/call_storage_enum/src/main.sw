script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x21ec4784feb8a4feda42fd1ccfb6c2496d42e03ff54f88be00602086491e1f7b;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
