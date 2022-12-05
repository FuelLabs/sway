script;

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
    __log(k);
    __log(42);
    __log(42u32);
    __log(42u16);
    __log(42u8);
    __log(a);
    __log(b);
    __log(test_struct);
    __log(test_enum);
}

// ::check-ir::

// check: script {
// check: fn main() -> ()
// check: entry():

// check: log b256 $VAL, $VAL
// check: log u64 $VAL, $VAL
// check: log u64 $VAL, $VAL
// check: log u64 $VAL, $VAL
// check: log u64 $VAL, $VAL
// check: log string<4> $VAL, $VAL
// check: log [u64; 3] $VAL, $VAL
// check: log { bool, b256, u64 } $VAL, $VAL
// check: log { u64 } $VAL, $VAL
