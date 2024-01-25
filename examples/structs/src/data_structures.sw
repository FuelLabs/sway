// the _data_structures_ module
library;

// Declare a struct type
pub struct Foo {
    pub bar: u64,
    pub baz: bool,
}

// Struct types for destructuring
pub struct Point {
    pub x: u64,
    pub y: u64,
}

pub struct Line {
    pub p1: Point,
    pub p2: Point,
}

pub struct TupleInStruct {
    pub nested_tuple: (u64, (u32, (bool, str))),
}

// Struct type instantiable only in the module _data_structures_
pub struct StructWithPrivateFields {
    pub public_field: u64,
    private_field: u64,
    other_private_field: u64,
}
