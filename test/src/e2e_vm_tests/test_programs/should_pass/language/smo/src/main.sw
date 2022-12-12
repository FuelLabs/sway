script;

fn smo<T>(recipient: b256, value: T, output_index: u64, coins: u64) {
    __smo::<T>(recipient, value, output_index, coins);
}

struct TestStruct<T> {
    field_1: bool,
    field_2: T,
    field_3: u64,
}

enum TestEnum {
    VariantOne: (),
    VariantTwo: (),
}

pub enum Option<T> {
    None: (),
    Some: T,
}

fn main() -> bool {
    let recipient = 0x0101010101010101010101010101010101010101010101010101010101010101;
    let output_index = 3;
    let coins = 24;

    // Check various data types as message data in `__smo`
    let k: b256 = 0xef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a;
    let a: str[4] = "Fuel";
    let b: [u8; 3] = [1u8, 2u8, 3u8];
    let test_struct = TestStruct {
        field_1: true,
        field_2: k,
        field_3: 11,
    };

    let test_enum = TestEnum::VariantTwo;
    smo(recipient, k, output_index, coins);
    smo(recipient, 42, output_index, coins);
    smo(recipient, 42u32, output_index, coins);
    smo(recipient, 42u16, output_index, coins);
    smo(recipient, 42u8, output_index, coins);
    __smo(recipient, a, output_index, coins);
    __smo(recipient, b, output_index, coins);
    __smo(recipient, test_struct, output_index, coins);
    __smo(recipient, test_enum, output_index, coins);
    __smo::<Option::<TestStruct<u64>>>(recipient, Option::Some(TestStruct {
        field_1: true,
        field_2: 42,
        field_3: 42,
    }), output_index, coins);

    // Make sure that logs don't clobber messages in the JSON ABI
    __log(a);
    __log(b);

    true
}
