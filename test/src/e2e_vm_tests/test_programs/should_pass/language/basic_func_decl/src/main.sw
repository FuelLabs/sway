script;
// This test tests function declarations and applications.

fn main() -> bool {
    let my_struct = MyStruct {
        a: 5,
    };
    let my_enum = MyEnum::Number(10);
    let my_struct_with_enum = MyStructWithEnum {
        a: my_struct,
        b: my_enum,
    };
    let d = "abcde";
    let e = true;
    let f = 15;
    let g = 0b10101010;
    let h: b256 = 0b1010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010101010;

    eight_args(my_struct, my_enum, my_struct_with_enum, d, e, f, g, h);

    // test some comparisons
    let ls_than = 4 < 5;
    let gt_than = 5 > 4;
    let le = 4 <= 4;
    let ge = 4 >= 4;
    let eq = 5 == 5;

    return true;
}
struct MyStruct {
    a: u64,
}

enum MyEnum {
    Number: u64,
    Unit: (),
}

struct MyStructWithEnum {
    a: MyStruct,
    b: MyEnum,
}

fn eight_args(a: MyStruct, b: MyEnum, c: MyStructWithEnum, d: str[5], e: bool, f: u64, g: u8, h: b256) {
    return;
}
