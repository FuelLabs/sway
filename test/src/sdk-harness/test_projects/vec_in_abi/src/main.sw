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
    fn bool_test(input: Vec<bool>) -> [bool; 3];
    fn u8_test(input: Vec<u8>) -> [u8; 3];
    fn u16_test(input: Vec<u16>) -> [u16; 3];
    fn u32_test(input: Vec<u32>) -> [u32; 3];
    fn u64_test(input: Vec<u64>) -> [u64; 3];
    fn b256_test(input: Vec<b256>) -> [b256; 3];
    fn struct_test(input: Vec<MyStruct>) -> [MyStruct; 3];
    fn enum_test(input: Vec<MyEnum>) -> [MyEnum; 3];
    fn array_test(input: Vec<[Address; 2]>) -> [[Address; 2]; 3];
    fn string_test(input: Vec<str[4]>) -> [str[4]; 3];
    fn vec_in_vec_test(input: Vec<Vec<u64>>) -> [u64; 9];
}

impl MyContract for Contract {
    fn bool_test(input: Vec<bool>) -> [bool; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn u8_test(input: Vec<u8>) -> [u8; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn u16_test(input: Vec<u16>) -> [u16; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn u32_test(input: Vec<u32>) -> [u32; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn u64_test(input: Vec<u64>) -> [u64; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn b256_test(input: Vec<b256>) -> [b256; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn struct_test(input: Vec<MyStruct>) -> [MyStruct; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn enum_test(input: Vec<MyEnum>) -> [MyEnum; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn array_test(input: Vec<[Address; 2]>) -> [[Address; 2]; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn string_test(input: Vec<str[4]>) -> [str[4]; 3] {
        assert(input.len() >= 3);
        [
            input.get(0).unwrap(),
            input.get(1).unwrap(),
            input.get(2).unwrap(),
        ]
    }
    fn vec_in_vec_test(input: Vec<Vec<u64>>) -> [u64; 9] {
        assert(input.len() >= 3);
        let v0 = input.get(0).unwrap();
        assert(v0.len() >= 3);
        let (v00, v01, v02) = (v0.get(0).unwrap(), v0.get(1).unwrap(), v0.get(2).unwrap());
        let v1 = input.get(1).unwrap();
        assert(v1.len() >= 3);
        let (v10, v11, v12) = (v1.get(0).unwrap(), v1.get(1).unwrap(), v1.get(2).unwrap());
        let v2 = input.get(2).unwrap();
        assert(v2.len() >= 3);
        let (v20, v21, v22) = (v2.get(0).unwrap(), v2.get(1).unwrap(), v2.get(2).unwrap());
        [v00, v01, v02, v10, v11, v12, v20, v21, v22]
    }
}
