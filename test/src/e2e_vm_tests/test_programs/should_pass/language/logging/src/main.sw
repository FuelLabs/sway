script;

fn log<T>(value: T) {
    __log::<T>(value);
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
    let k: b256 = 0xef86afa9696cf0dc6385e2c407a6e159a1103cefb7e2ae0636fb33d3cb2a9e4a;
    let a: str = "Fuel";
    let b: [u8; 3] = [1u8, 2u8, 3u8];
    let test_struct = TestStruct {
        field_1: true,
        field_2: k,
        field_3: 11,
    };

    let test_enum = TestEnum::VariantTwo;
    log(k);
    log(42);
    log(42u32);
    log(42u16);
    log(42u8);
    __log(a);
    __log(b);
    __log(test_struct);
    __log(test_enum);
    __log(Some(TestStruct {
        field_1: true,
        field_2: 42,
        field_3: 42,
    }));

    true
}
