contract;

configurable {
    BOOL: bool = true,
    U8: u8 = 8,
    U16: u16 = 16,
    U32: u32 = 32,
    U64: u64 = 63,
    U256: u256 = 0x0000000000000000000000000000000000000000000000000000000000000008u256,
    B256: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101,
    STR_4: str[4] = __to_str_array("fuel"),
}

abi TestContract {
    fn return_configurables() -> (bool, u8, u16, u32, u64, u256, b256, str[4]);
}

impl TestContract for Contract {
    fn return_configurables() -> (bool, u8, u16, u32, u64, u256, b256, str[4]) {
        (BOOL, U8, U16, U32, U64, U256, B256, STR_4)
    }
}
