script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0x4c01b41e6f7fc88c88a7799c43d9f695e22ee01eed90478b99fe3bfa935e3e07;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x026657ad76db31e965b71751d10cef56ecb2fae762f0e655d0da54827ccd120e;

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
