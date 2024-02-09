script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x2c6686f3a059e41298f5680c92a8effdc628cf86ac293b84ea9fc10fa1fd7906;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
