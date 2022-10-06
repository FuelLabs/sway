script;

use std::logging::log;

struct TestStruct {
    field_1: bool,
    field_2: b256,
    field_3: u64,
}

enum TestEnum {
    VariantOne: (),
    VariantTwo: (),
}

fn main() {
    let k: b256 = 0xef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a;
    let a: str[4] = "Fuel";
    let b: [u8; 3] = [1u8, 2u8, 3u8];
    let test_struct = TestStruct {
        field_1: true,
        field_2: k,
        field_3: 11,
    };

    let test_enum = TestEnum::VariantTwo;

    log(k);
    log(42u64);
    log(42u32);
    log(42u16);
    log(42u8);
    log(a);
    log(b);
    log(test_struct);
    log(test_enum);
}
