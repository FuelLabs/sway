script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x00aafec257cdec485dd02d8e6e2ef9fdba8a71c34817df5ed66d117a67a2741d;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
