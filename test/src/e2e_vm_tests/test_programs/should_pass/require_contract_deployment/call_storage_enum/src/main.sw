script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x039f59a5f7ab74f3c75eedaedeabdbff9b8bc5310f44ff10b0344fc316026e7d;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x279b83091be356db25e04ff4bbf5aea890fd8eca26a441f99575e8b8e6672daf;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
