library;

pub struct Foo {}
impl Foo {
    const MIN: Self = Self {};
}

impl Foo {
    pub fn foo() {
        let x = Self::MIN;
    }
}