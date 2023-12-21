library;

pub enum Enum {
    A: (),
}

pub struct Struct {
    x: u64,
}

pub struct PubStruct {
   x: u64,
}

pub struct GenericStruct<T> {
    x: T,
}

pub const X: u64 = 0u64;