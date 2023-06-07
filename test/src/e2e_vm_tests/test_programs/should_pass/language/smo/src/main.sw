script;

fn smo<T>(recipient: b256, value: T, coins: u64) {
    __smo::<T>(recipient, value, coins);
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
    smo(recipient, k, coins);
    smo(recipient, 42, coins);
    smo(recipient, 42u32, coins);
    smo(recipient, 42u16, coins);
    smo(recipient, 42u8, coins);
    __smo(recipient, a, coins);
    __smo(recipient, b, coins);
    __smo(recipient, test_struct, coins);
    __smo(recipient, test_enum, coins);
    __smo::<Option::<TestStruct<u64>>>(recipient, Option::Some(TestStruct {
        field_1: true,
        field_2: 42,
        field_3: 42,
    }), coins);

    // Make sure that logs don't clobber messages in the JSON ABI
    __log(a);
    __log(b);

    true
}
