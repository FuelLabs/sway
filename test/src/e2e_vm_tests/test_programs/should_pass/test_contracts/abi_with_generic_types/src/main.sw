contract;

enum MyEnum<V> {
    Foo: u64,
    Bar: bool,
}
struct MyStruct<T, U> {
    bim: T,
    bam: MyEnum<u64>,
}
struct MyOtherStruct {
    bom: u64,
}

struct MyArrayStruct<V, W> {
    tim: [V;
    3],
    tam: [MyStruct<V,
    W>;
    5], 
}

abi MyContract {
    fn complex_function(arg1: MyStruct<[b256;
    3], u8>, arg2: [MyStruct<u64, bool>;
    4], arg3: (str[5], bool), arg4: MyOtherStruct, ) -> str[6];
    fn take_generic_array(arg: MyArrayStruct<u8, u16>) -> u64;
}

impl MyContract for Contract {
    fn complex_function(arg1: MyStruct<[b256;
    3], u8>, arg2: [MyStruct<u64, bool>;
    4], arg3: (str[5], bool), arg4: MyOtherStruct, ) -> str[6] {
        "fuel42"
    }
    fn take_generic_array(arg: MyArrayStruct<u8, u16>) -> u64 {
        0
    }
}
