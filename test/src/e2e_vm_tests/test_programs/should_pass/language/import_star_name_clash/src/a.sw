library;

// Allow dead code because of a bug in the dead code elimination
// See https://github.com/FuelLabs/sway/issues/5902
#[allow(dead_code)]
pub struct MyStruct {
    pub a: u64,
}

pub enum MyEnum {
    A: u64,
    B: u64,
}

pub struct C {
    pub b: u64,
}
