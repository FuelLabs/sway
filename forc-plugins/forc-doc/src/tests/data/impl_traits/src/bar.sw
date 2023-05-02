library;

use ::foo::Foo;

pub struct Bar {}

impl Foo for Bar {
    /// somethings more about foo();
    fn foo() {}
}
impl Bar {
    fn foo_bar() {
        Self::foo()
    }
}