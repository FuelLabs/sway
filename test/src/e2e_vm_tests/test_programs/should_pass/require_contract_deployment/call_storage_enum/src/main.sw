script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x039f59a5f7ab74f3c75eedaedeabdbff9b8bc5310f44ff10b0344fc316026e7d;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
