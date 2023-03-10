contract;

enum EnumWithGeneric<D> {
    VariantOne: D,
    VariantTwo: (),
}

struct StructWithGeneric<D> {
    field_1: D,
    field_2: u64,
}

// ANCHOR: configurable_block
configurable {
    U8: u8 = 8u8,
    BOOL: bool = true,
    ARRAY: [u32; 3] = [253u32, 254u32, 255u32],
    STR_4: str[4] = "fuel",
    STRUCT: StructWithGeneric<u8> = StructWithGeneric {
        field_1: 8u8,
        field_2: 16,
    },
    ENUM: EnumWithGeneric<bool> = EnumWithGeneric::VariantOne(true),
}
// ANCHOR_END: configurable_block 

abi TestContract {
    fn return_configurables() -> (u8, bool, [u32; 3], str[4], StructWithGeneric<u8>);
}

impl TestContract for Contract {
// ANCHOR: using_configurables
    fn return_configurables() -> (u8, bool, [u32; 3], str[4], StructWithGeneric<u8>) {
        (U8, BOOL, ARRAY, STR_4, STRUCT)
    }
// ANCHOR_END: using_configurables
}
