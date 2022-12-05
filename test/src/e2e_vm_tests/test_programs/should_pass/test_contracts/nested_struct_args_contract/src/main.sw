contract;

use nested_struct_args_abi::*;

impl NestedStructArgs for Contract {
    fn foo(input1: StructOne, input2: StructTwo) -> u64 {
        let v = input1.inn.foo + input2.foo;
        v + 1
    }
}
