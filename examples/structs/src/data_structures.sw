library data_structures;

// Declare a struct type
pub struct Foo {
    bar: u64,
    baz: bool,
}

// Struct types for destructuring
pub struct Point {
    x: u64,
    y: u64,
}

pub struct Line {
    p1: Point,
    p2: Point,
}

pub struct TupleInStruct {
    nested_tuple: (u64, (u32, (bool, str[2]))),
}
