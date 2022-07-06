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

abi MyContract {
    fn complex_function(arg1: MyStruct<[b256;
    3], u8>, arg2: [MyStruct<u64, bool>;
    4], arg3: (str[5], bool), arg3: MyOtherStruct, ) -> str[6];
}

impl MyContract for Contract {
    fn complex_function(arg1: MyStruct<[b256;
    3], u8>, arg2: [MyStruct<u64, bool>;
    4], arg3: (str[5], bool), arg4: MyOtherStruct, ) -> str[6] {
        "fuel42"
    }
}
