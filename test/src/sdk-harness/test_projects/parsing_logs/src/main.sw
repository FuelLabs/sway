contract;

use core::codec::*;

struct TestStruct {
    field_1: bool,
    field_2: b256,
    field_3: u64,
}

enum TestEnum {
    VariantOne: (),
    VariantTwo: (),
}

impl AbiEncode for TestEnum {
    fn abi_encode(self, ref mut buffer: Buffer) {
        match self {
            TestEnum::VariantOne => {}
            TestEnum::VariantTwo => {}
        }
    }
}

struct StructWithGeneric<D> {
    field_1: D,
    field_2: u64,
}

//impl<D> AbiEncode for StructWithGeneric<D>
//where
//    D: AbiEncode
//{
//    fn abi_encode(self, ref mut buffer: Buffer) {
//        self.field_1.abi_encode(buffer);
//        self.field_2.abi_encode(buffer);
//    }
//}

enum EnumWithGeneric<D> {
    VariantOne: D,
    VariantTwo: (),
}

impl<D> AbiEncode for EnumWithGeneric<D>
where
    D: AbiEncode
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        match self {
            EnumWithGeneric::VariantOne(x) => x.abi_encode(buffer),
            EnumWithGeneric::VariantTwo => {}
        }
    }
}

struct StructWithNestedGeneric<D> {
    field_1: D,
    field_2: u64,
}

impl<D> AbiEncode for StructWithNestedGeneric<D>
where
    D: AbiEncode
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.field_1.abi_encode(buffer);
        self.field_2.abi_encode(buffer);
    }
}

struct StructDeeplyNestedGeneric<D> {
    field_1: D,
    field_2: u64,
}

impl<D> AbiEncode for StructDeeplyNestedGeneric<D>
where
    D: AbiEncode
{
    fn abi_encode(self, ref mut buffer: Buffer) {
        self.field_1.abi_encode(buffer);
        self.field_2.abi_encode(buffer);
    }
}

abi TestContract {
    fn produce_logs_values() -> ();
    fn produce_logs_variables() -> ();
    fn produce_logs_custom_types() -> ();
    fn produce_logs_generic_types() -> ();
    fn produce_multiple_logs() -> ();
}

impl TestContract for Contract {
    fn produce_logs_values() {
        log(64);
        log(32u32);
        log(16u16);
        log(8u8);
    }

    fn produce_logs_variables() -> () {
        let f: u64 = 64;
        let u: b256 = 0xef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a;
        let e: str[4] = __to_str_array("Fuel");
        let l: [u8; 3] = [1u8, 2u8, 3u8];

        log(f);
        log(u);
        log(e);
        log(l);
    }

    fn produce_logs_custom_types() -> () {
        let f: u64 = 64;
        let u: b256 = 0xef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a;

        let test_struct = TestStruct {
            field_1: true,
            field_2: u,
            field_3: f,
        };
        let test_enum = TestEnum::VariantTwo;

        log(test_struct);
        log(test_enum);
    }

    fn produce_logs_generic_types() -> () {
        let l: [u8; 3] = [1u8, 2u8, 3u8];

        let test_struct = StructWithGeneric {
            field_1: l,
            field_2: 64,
        };
        let test_enum = EnumWithGeneric::VariantOne(l);
        let test_struct_nested = StructWithNestedGeneric {
            field_1: test_struct,
            field_2: 64,
        };
        let test_deeply_nested_generic = StructDeeplyNestedGeneric {
            field_1: test_struct_nested,
            field_2: 64,
        };

        log(test_struct);
        log(test_enum);
        log(test_struct_nested);
        log(test_deeply_nested_generic);
    }

    fn produce_multiple_logs() -> () {
        let f: u64 = 64;
        let u: b256 = 0xef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a;
        let e: str[4] = __to_str_array("Fuel");
        let l: [u8; 3] = [1u8, 2u8, 3u8];
        let test_struct = TestStruct {
            field_1: true,
            field_2: u,
            field_3: f,
        };
        let test_enum = TestEnum::VariantTwo;
        let test_generic_struct = StructWithGeneric {
            field_1: test_struct,
            field_2: 64,
        };

        log(64);
        log(32u32);
        log(16u16);
        log(8u8);
        log(f);
        log(u);
        log(e);
        log(l);
        log(test_struct);
        log(test_enum);
        log(test_generic_struct);
    }
}
