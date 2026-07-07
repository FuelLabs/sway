script;

use storage_enum_abi::*;

#[cfg(experimental_new_encoding = false)]
const CONTRACT_ID = 0xc601d11767195485a6654d566c67774134668863d8c797a8c69e8778fb1f89e9;
#[cfg(experimental_new_encoding = true)]
const CONTRACT_ID = 0x56079fbbd98a02a92e73ad6f031b2d5317e54a54551796c2961f4c90d508c1e3; // AUTO-CONTRACT-ID ../../test_contracts/storage_enum_contract --release
fn main() -> u64 {
    let caller = abi(StorageEnum, CONTRACT_ID);
    let res = caller.read_write_enums();
    res
}
