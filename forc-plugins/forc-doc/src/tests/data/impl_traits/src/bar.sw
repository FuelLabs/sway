library;

use ::foo::{Foo, Baz};

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