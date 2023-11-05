script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x8836ec87c97907ff0f4cf0cf62f852c7f654b8a6bd6845ee2d0203c5dbd029a5;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
