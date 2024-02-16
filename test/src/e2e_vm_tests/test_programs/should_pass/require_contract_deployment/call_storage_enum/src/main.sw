script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x490b0ac1da56e77c5d5c1efd2fd1f4294a683ab039697eacf913b796a4d59196;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
