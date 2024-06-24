script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xb54eb67f943fda15981308ef55b2cb2afe1ed5907c483d2d92d010ab39549644;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xfd91339494a1600ffb24d90eb7b67582e11fee5936e01704fb17a5445eb3f47e;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
