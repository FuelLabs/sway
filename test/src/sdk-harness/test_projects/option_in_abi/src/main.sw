contract;

pub enum SomeError {
    SomeErrorString: str[5],
}

struct MyStruct {
    first_field: Option<Address>,
    second_field: u64,
}

enum MyEnum {
    FirstVariant: Option<Address>,
    SecondVariant: u64,
}

abi MyContract {
    fn bool_test(input: Option<bool>) -> Option<bool>;
    fn u8_test(input: Option<u8>) -> Option<u8>;
    fn u16_test(input: Option<u16>) -> Option<u16>;
    fn u32_test(input: Option<u32>) -> Option<u32>;
    fn u64_test(input: Option<u64>) -> Option<u64>;
    fn b256_test(input: Option<b256>) -> Option<b256>;
    fn struct_test(input: Option<MyStruct>) -> Option<MyStruct>;
    fn tuple_test(input: Option<(Option<Address>, u64)>) -> Option<(Option<Address>, u64)>;
    fn enum_test(input: Option<MyEnum>) -> Option<MyEnum>;
    fn array_test(input: Option<[Option<Address>; 3]>) -> Option<[Option<Address>; 3]>;
    fn string_test(input: Option<str[4]>) -> Option<str[4]>;
    fn result_in_option_test(input: Option<Result<str[4], SomeError>>) -> Option<Result<str[4], SomeError>>;
}

impl MyContract for Contract {
    fn bool_test(input: Option<bool>) -> Option<bool> {
        input
    }
    fn u8_test(input: Option<u8>) -> Option<u8> {
        input
    }
    fn u16_test(input: Option<u16>) -> Option<u16> {
        input
    }
    fn u32_test(input: Option<u32>) -> Option<u32> {
        input
    }
    fn u64_test(input: Option<u64>) -> Option<u64> {
        input
    }
    fn b256_test(input: Option<b256>) -> Option<b256> {
        input
    }
    fn struct_test(input: Option<MyStruct>) -> Option<MyStruct> {
        input
    }
    fn tuple_test(input: Option<(Option<Address>, u64)>) -> Option<(Option<Address>, u64)> {
        input
    }
    fn enum_test(input: Option<MyEnum>) -> Option<MyEnum> {
        input
    }
    fn array_test(input: Option<[Option<Address>; 3]>) -> Option<[Option<Address>; 3]> {
        input
    }
    fn string_test(input: Option<str[4]>) -> Option<str[4]> {
        input
    }
    fn result_in_option_test(input: Option<Result<str[4], SomeError>>) -> Option<Result<str[4], SomeError>> {
        input
    }
}
