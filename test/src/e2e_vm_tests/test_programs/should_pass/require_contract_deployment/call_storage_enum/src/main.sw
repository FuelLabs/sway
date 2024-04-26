script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x1ce765336fbb4c4558c7f5753fad01a8549521b03e82bc94048fa512750b9554;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x2126d89a1de460b674af6d1d956051272f5b54bef7472be87c34f53b88ecd8d3;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
