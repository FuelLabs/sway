script;

struct MyStruct {
    x: u64,
}

enum MyEnum {
    A: u64,
    B: bool,
}

pub type StringArray = str[4];
pub type Array = [u32; 2];

impl PartialEq for MyStruct {
    fn eq(self, other: Self) -> bool {
        self.x == other.x
    }
}
impl Eq for MyStruct {}

pub type Tuple = (u32, u32);

impl PartialEq for MyEnum {
    fn eq(self, other: MyEnum) -> bool {
        match (self, other) {
            (MyEnum::A(inner1), MyEnum::A(inner2)) => inner1 == inner2,
            (MyEnum::B(inner1), MyEnum::B(inner2)) => inner1 == inner2,
            _ => false,
        }
    }
}
impl Eq for MyEnum {}

fn main() -> bool {
    // Test with u8
    let u8_option1 = Option::<u8>::Some(10);
    let u8_option2 = Option::<u8>::Some(10);
    let u8_option3 = Option::<u8>::Some(20);
    let u8_none_option: Option<u8> = Option::None;

    // Eq tests
    assert(u8_option1 == u8_option1);
    assert(u8_option1 == u8_option2);

    // Neq tests
    assert(u8_option1 != u8_option3);
    assert(u8_option1 != u8_none_option);

    // None tests
    assert(u8_none_option == Option::None);
    assert(Option::<u8>::None == u8_none_option);

    // Test with u16
    let u16_option1 = Option::<u16>::Some(10);
    let u16_option2 = Option::<u16>::Some(10);
    let u16_option3 = Option::<u16>::Some(20);
    let u16_none_option: Option<u16> = Option::None;

    // Eq tests
    assert(u16_option1 == u16_option1);
    assert(u16_option1 == u16_option2);

    // Neq tests
    assert(u16_option1 != u16_option3);
    assert(u16_option1 != u16_none_option);

    // None tests
    assert(u16_none_option == Option::None);
    assert(Option::<u16>::None == u16_none_option);

    // Test with u32
    let u32_option1 = Option::<u32>::Some(10);
    let u32_option2 = Option::<u32>::Some(10);
    let u32_option3 = Option::<u32>::Some(20);
    let u32_none_option: Option<u32> = Option::None;

    // Eq tests
    assert(u32_option1 == u32_option1);
    assert(u32_option1 == u32_option2);

    // Neq tests
    assert(u32_option1 != u32_option3);
    assert(u32_option1 != u32_none_option);

    // None tests
    assert(u32_none_option == Option::None);
    assert(Option::<u32>::None == u32_none_option);

    // Test with u64
    let u64_option1 = Option::<u64>::Some(10);
    let u64_option2 = Option::<u64>::Some(10);
    let u64_option3 = Option::<u64>::Some(20);
    let u64_none_option: Option<u64> = Option::None;

    // Eq tests
    assert(u64_option1 == u64_option1);
    assert(u64_option1 == u64_option2);

    // Neq tests
    assert(u64_option1 != u64_option3);
    assert(u64_option1 != u64_none_option);

    // None tests
    assert(u64_none_option == Option::None);
    assert(Option::<u64>::None == u64_none_option);

    // Test with u256
    let u256_option1 = Option::<u256>::Some(10);
    let u256_option2 = Option::<u256>::Some(10);
    let u256_option3 = Option::<u256>::Some(20);
    let u256_none_option: Option<u256> = Option::None;

    // Eq tests
    assert(u256_option1 == u256_option1);
    assert(u256_option1 == u256_option2);

    // Neq tests
    assert(u256_option1 != u256_option3);
    assert(u256_option1 != u256_none_option);

    // None tests
    assert(u256_none_option == Option::None);
    assert(Option::<u256>::None == u256_none_option);

    // Test with str
    let str_option1 = Option::<str>::Some("fuel");
    let str_option2 = Option::<str>::Some("fuel");
    let str_option3 = Option::<str>::Some("sway");
    let str_none_option: Option<str> = Option::None;

    // Eq tests
    assert(str_option1 == str_option1);
    assert(str_option1 == str_option2);

    // Neq tests
    assert(str_option1 != str_option3);
    assert(str_option1 != str_none_option);

    // None tests
    assert(str_none_option == Option::None);
    assert(Option::<str>::None == str_none_option);

    // Test with bool
    let bool_option1 = Option::Some(true);
    let bool_option2 = Option::Some(true);
    let bool_option3 = Option::Some(false);
    let bool_none_option: Option<bool> = Option::None;

    // Eq tests
    assert(bool_option1 == bool_option1);
    assert(bool_option1 == bool_option2);

    // Neq tests
    assert(bool_option1 != bool_option3);
    assert(bool_option1 != bool_none_option);

    // None tests
    assert(bool_none_option == Option::None);
    assert(Option::<bool>::None == bool_none_option);

    // Test with b256
    let b256_option1 = Option::<b256>::Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_option2 = Option::<b256>::Some(0x0000000000000000000000000000000000000000000000000000000000000001);
    let b256_option3 = Option::<b256>::Some(0x0000000000000000000000000000000000000000000000000000000000000002);
    let b256_none_option: Option<b256> = Option::None;

    // Eq tests
    assert(b256_option1 == b256_option1);
    assert(b256_option1 == b256_option2);

    // Neq tests
    assert(b256_option1 != b256_option3);
    assert(b256_option1 != b256_none_option);

    // None tests
    assert(b256_none_option == Option::None);
    assert(Option::<b256>::None == b256_none_option);

    // Test with string array
    let string1: StringArray = __to_str_array("fuel");
    let string2: StringArray = __to_str_array("sway");

    let string_option1 = Option::Some(string1);
    let string_option2 = Option::Some(string1);
    let string_option3 = Option::Some(string2);
    let string_none_option: Option<StringArray> = Option::None;

    // Eq tests
    assert(string_option1 == string_option1);
    assert(string_option1 == string_option2);

    // Neq tests
    assert(string_option1 != string_option3);
    assert(string_option1 != string_none_option);

    // None tests
    assert(string_none_option == Option::None);
    assert(Option::<StringArray>::None == string_none_option);

    // Test with array
    let array1: Array = [10, 20];
    let array2: Array = [10, 30];

    let array_option1 = Option::Some(array1);
    let array_option2 = Option::Some(array1);
    let array_option3 = Option::Some(array2);
    let array_none_option: Option<Array> = Option::None;

    // Eq tests
    assert(array_option1 == array_option1);
    assert(array_option1 == array_option2);

    // Neq tests
    assert(array_option1 != array_option3);
    assert(array_option1 != array_none_option);

    // None tests
    assert(array_none_option == Option::None);
    assert(Option::<Array>::None == array_none_option);

    // Test with struct
    let struct_option1 = Option::Some(MyStruct { x: 10 });
    let struct_option2 = Option::Some(MyStruct { x: 10 });
    let struct_option3 = Option::Some(MyStruct { x: 20 });
    let struct_none_option: Option<MyStruct> = Option::None;

    // Eq tests
    assert(struct_option1 == struct_option1);
    assert(struct_option1 == struct_option2);

    // Neq tests
    assert(struct_option1 != struct_option3);
    assert(struct_option1 != struct_none_option);

    // None tests
    assert(struct_none_option == Option::None);
    assert(Option::<MyStruct>::None == struct_none_option);

    // Test with tuple
    let tuple1: Tuple = (10, 20);
    let tuple2: Tuple = (10, 30);

    let tuple_option1 = Option::Some(tuple1);
    let tuple_option2 = Option::Some(tuple1);
    let tuple_option3 = Option::Some(tuple2);
    let tuple_none_option: Option<Tuple> = Option::None;

    // Eq tests
    assert(tuple_option1 == tuple_option1);
    assert(tuple_option1 == tuple_option2);

    // Neq tests
    assert(tuple_option1 != tuple_option3);
    assert(tuple_option1 != tuple_none_option);

    // None tests
    assert(tuple_none_option == Option::None);
    assert(Option::<Tuple>::None == tuple_none_option);

    // Test with enums
    let enum_option1 = Option::Some(MyEnum::A(42));
    let enum_option2 = Option::Some(MyEnum::A(42));
    let enum_option3 = Option::Some(MyEnum::B(true));
    let enum_none_option: Option<MyEnum> = Option::None;

    // Eq tests
    assert(enum_option1 == enum_option1);
    assert(enum_option1 == enum_option2);

    // Neq tests
    assert(enum_option1 != enum_option3);
    assert(enum_option1 != enum_none_option);

    // None tests
    assert(enum_none_option == Option::None);
    assert(Option::<MyEnum>::None == enum_none_option);

    true
}
