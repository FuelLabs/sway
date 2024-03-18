script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x039f59a5f7ab74f3c75eedaedeabdbff9b8bc5310f44ff10b0344fc316026e7d;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0xf5a4eff97a1729dfaaac2cdf8e4d05162013e21a347c66a47495cc9faf122b42;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
