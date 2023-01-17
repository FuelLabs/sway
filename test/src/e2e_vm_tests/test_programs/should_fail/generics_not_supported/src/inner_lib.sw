library inner_lib;

const C = 42;

pub enum MyEnum<T> {
    VariantA: (),
    VariantB: T
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