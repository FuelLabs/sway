library;

struct EmptyStruct { }

struct Struct {
    x: u8,
}

enum EmptyEnum {}

enum MyEnum {
    A: (),
}

// Implement `Enum`.
impl Enum for MyEnum { }

impl Enum for EmptyEnum { }

impl Enum for Struct { }

impl Enum for EmptyStruct {
    fn non_existing() {}
}

impl Enum for [u64;0] { }

impl core::marker::Enum for (u8, u16, u32, u64, u256) { }

// Implement `Error`.
impl Error for MyEnum { }

impl Error for EmptyEnum { }

impl Error for Struct { }

impl Error for EmptyStruct {
    fn non_existing() {}
}

impl Error for [u64;0] { }

impl core::marker::Error for (u8, u16, u32, u64, u256) { }
