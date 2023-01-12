library inner_lib;

pub enum MyEnum {
    VariantA: (),
    VariantB: u64
}

pub fn func() -> bool {
    true
}

pub struct S2 {}

impl S2 {
    pub fn new2() -> Self {
        S2 {}
    }
}