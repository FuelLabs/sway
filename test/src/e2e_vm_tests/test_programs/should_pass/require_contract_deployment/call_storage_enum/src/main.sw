script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0xe2f370fb01c8c36a521093494a023ce2d7232398e5e88722164c42ec0303f381;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
