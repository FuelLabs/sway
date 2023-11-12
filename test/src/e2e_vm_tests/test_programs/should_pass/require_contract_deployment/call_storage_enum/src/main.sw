script;

use storage_enum_abi::*;

fn main() -> u64 {
    let contract_id = 0x2242ad80dd0a53dc5eb440051bbe010f4817125774c82f0122d315df57338dc8;
    let caller = abi(StorageEnum, contract_id);

    let res = caller.read_write_enums();

    res
}
