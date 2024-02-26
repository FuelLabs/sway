script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x68e2d67cc9ba503882c931ad2425d9fe91bfe78b1b59d039a6e2f06e2c9ce130;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
