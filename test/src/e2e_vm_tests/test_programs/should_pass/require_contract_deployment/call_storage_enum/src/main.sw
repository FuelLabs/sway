script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID: b256 = 0xc601d11767195485a6654d566c67774134668863d8c797a8c69e8778fb1f89e9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID: b256 = 0x9ff1242ab03642c1977b8a1d3b6fe8dd0fd2780cf039ed75e6f1503cefece8f8; // AUTO-CONTRACT-ID ../../test_contracts/storage_enum_contract --release

fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
