script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x039f59a5f7ab74f3c75eedaedeabdbff9b8bc5310f44ff10b0344fc316026e7d;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x2efef9b3a025bfe8acc8996c8b9b5547f8f0f89e53d0b4754950b1ad84e21b4e;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
