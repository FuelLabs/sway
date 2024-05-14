library;

pub struct Inner {
    pub foo: u64
}

pub struct StructOne {
    pub inn: Inner,
}

pub struct StructTwo {
    pub foo: u64,
}

abi NestedStructArgs {
    fn foo(input1: StructOne, input2: StructTwo) -> u64;
}
