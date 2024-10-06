script;

use std::logging::log;

#[allow(dead_code)]
enum EnumWithGeneric<D> {
    VariantOne: D,
    VariantTwo: (),
}

struct StructWithGeneric<D> {
    field_1: D,
    field_2: u64,
}

abi MyContract {
    fn test_function() -> bool;

    #[storage(read)]
    fn test_function_read() -> u8;

    #[storage(read, write)]
    fn test_function_write(value: u8) -> u8;
}

configurable {
    BOOL: bool = true,
    U8: u8 = 8,
    U16: u16 = 16,
    U32: u32 = 32,
    U64: u64 = 63,
    U256: u256 = 0x0000000000000000000000000000000000000000000000000000000000000008u256,
    B256: b256 = 0x0101010101010101010101010101010101010101010101010101010101010101,
    STR_4: str[4] = __to_str_array("fuel"),
    TUPLE: (u8, bool) = (8, true),
    ARRAY: [u32; 3] = [253, 254, 255],
    STRUCT: StructWithGeneric<u8> = StructWithGeneric {
        field_1: 8,
        field_2: 16,
    },
    ENUM: EnumWithGeneric<bool> = EnumWithGeneric::VariantOne(true),
}

fn get_configurables() -> (bool, u8, u16, u32, u64, u256, b256, str[4], (u8, bool), [u32; 3], StructWithGeneric<u8>, EnumWithGeneric<bool>) {
    (BOOL, U8, U16, U32, U64, U256, B256, STR_4, TUPLE, ARRAY, STRUCT, ENUM)
}

fn basic_function_with_input(a: u32) -> bool {
    if a % 2 == 0 {
        true
    }else {
        false
    }
}

fn basic_function_without_input() -> u64 {
    let a = 100;
    let b = 25;
    a*b
}

fn main(a: u32, contract_addr: b256) -> ((bool, u8, u16, u32, u64, u256, b256, str[4], (u8, bool), [u32; 3], StructWithGeneric<u8>, EnumWithGeneric<bool>), bool, u64, u8) {
    log(U8);
    let configs = get_configurables();
    let with_in = basic_function_with_input(a);
    let without_in = basic_function_without_input();

    let contract_instance = abi(MyContract, contract_addr);
    let from_contract = contract_instance.test_function_read();

    return (configs, with_in, without_in, from_contract);
}
