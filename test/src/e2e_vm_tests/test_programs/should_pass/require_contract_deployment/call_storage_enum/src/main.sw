script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x98847a788b7eb370581f902c5395ba999d7d16b4762fc31f4719cbf88f2c1dcb;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
