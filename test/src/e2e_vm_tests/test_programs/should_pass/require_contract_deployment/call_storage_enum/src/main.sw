script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0xc74997f9aa1ca58769488bb24d7fef29e7cd99f75f640234eaec9cc4175adad5;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
