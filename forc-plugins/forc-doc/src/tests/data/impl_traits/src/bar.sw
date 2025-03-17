library;

use ::foo::{Foo, Baz};
use ops::Add;

pub struct Bar {}

impl Foo for Bar {
    /// something more about foo();
    fn foo() {}
}
impl Baz for Bar {}
impl Bar {
    fn foo_bar() {
        Self::foo()
    }
}

// test dependency impls
impl Add for Bar {
    fn add(self, other: Self) -> Self {
        Bar {}
    }
}
impl ops::Subtract for Bar {
    fn subtract(self, other: Self) -> Self {
        Bar {}
    }
}