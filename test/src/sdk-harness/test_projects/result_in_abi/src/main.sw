contract;

pub enum SomeError {
    SomeErrorString: str[5],
}

struct MyStruct {
    first_field: Result<Address, SomeError>,
    second_field: u64,
}

enum MyEnum {
    FirstVariant: Result<Address, SomeError>,
    SecondVariant: u64,
}

abi MyContract {
    fn bool_test(input: Result<bool, SomeError>) -> Result<bool, SomeError>;
    fn u8_test(input: Result<u8, SomeError>) -> Result<u8, SomeError>;
    fn u16_test(input: Result<u16, SomeError>) -> Result<u16, SomeError>;
    fn u32_test(input: Result<u32, SomeError>) -> Result<u32, SomeError>;
    fn u64_test(input: Result<u64, SomeError>) -> Result<u64, SomeError>;
    fn b256_test(input: Result<b256, SomeError>) -> Result<b256, SomeError>;
    fn struct_test(input: Result<MyStruct, SomeError>) -> Result<MyStruct, SomeError>;
    fn tuple_test(input: Result<(Result<Address, SomeError>, u64), SomeError>) -> Result<(Result<Address, SomeError>, u64), SomeError>;
    fn enum_test(input: Result<MyEnum, SomeError>) -> Result<MyEnum, SomeError>;
    fn array_test(input: Result<[Result<Address, SomeError>; 3], SomeError>) -> Result<[Result<Address, SomeError>; 3], SomeError>;
    fn string_test(input: Result<str[4], SomeError>) -> Result<str[4], SomeError>;
    fn option_in_result_test(input: Result<Option<str[4]>, SomeError>) -> Result<Option<str[4]>, SomeError>;
}

impl MyContract for Contract {
    fn bool_test(input: Result<bool, SomeError>) -> Result<bool, SomeError> {
        input
    }
    fn u8_test(input: Result<u8, SomeError>) -> Result<u8, SomeError> {
        input
    }
    fn u16_test(input: Result<u16, SomeError>) -> Result<u16, SomeError> {
        input
    }
    fn u32_test(input: Result<u32, SomeError>) -> Result<u32, SomeError> {
        input
    }
    fn u64_test(input: Result<u64, SomeError>) -> Result<u64, SomeError> {
        input
    }
    fn b256_test(input: Result<b256, SomeError>) -> Result<b256, SomeError> {
        input
    }
    fn struct_test(input: Result<MyStruct, SomeError>) -> Result<MyStruct, SomeError> {
        input
    }
    fn tuple_test(
        input: Result<(Result<Address, SomeError>, u64), SomeError>,
    ) -> Result<(Result<Address, SomeError>, u64), SomeError> {
        input
    }
    fn enum_test(input: Result<MyEnum, SomeError>) -> Result<MyEnum, SomeError> {
        input
    }
    fn array_test(
        input: Result<[Result<Address, SomeError>; 3], SomeError>,
    ) -> Result<[Result<Address, SomeError>; 3], SomeError> {
        input
    }
    fn string_test(input: Result<str[4], SomeError>) -> Result<str[4], SomeError> {
        input
    }
    fn option_in_result_test(input: Result<Option<str[4]>, SomeError>) -> Result<Option<str[4]>, SomeError> {
        input
    }
}
