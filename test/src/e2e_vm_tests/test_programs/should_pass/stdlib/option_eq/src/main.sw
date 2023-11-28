script;

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

    true
}
