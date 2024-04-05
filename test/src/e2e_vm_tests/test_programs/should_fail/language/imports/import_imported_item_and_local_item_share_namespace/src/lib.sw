library;

pub enum Enum {
    A: (),
}

pub struct Struct {
    pub x: u64,
}

pub struct PubStruct {
    pub x: u64,
}

pub struct GenericStruct<T> {
    pub x: T,
}

pub const X: u64 = 0u64;