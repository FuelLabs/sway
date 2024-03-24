script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xb39368a42dec58dabead50a7c97953d3a49fcf35d63a80378240d36538c99745;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x18625bd7d24adb1da41be8bd4a51c15d8657ff66d37b3d7086601cddb2e7e100;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
