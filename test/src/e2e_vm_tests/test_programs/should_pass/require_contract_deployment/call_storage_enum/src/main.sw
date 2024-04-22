script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x1ce765336fbb4c4558c7f5753fad01a8549521b03e82bc94048fa512750b9554;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x35edb2feb79541369f9d4353b39ffbb044a95a1944e219f431e5135275e6d434;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
