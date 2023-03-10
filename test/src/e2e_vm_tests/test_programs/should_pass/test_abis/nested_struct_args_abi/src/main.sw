library;

pub struct Inner {
    foo: u64
}

pub struct StructOne {
    inn: Inner,
}

pub struct StructTwo {
    foo: u64,
}

abi NestedStructArgs {
    fn foo(input1: StructOne, input2: StructTwo) -> u64;
}
