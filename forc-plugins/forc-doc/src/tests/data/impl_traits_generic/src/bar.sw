library;

use ::foo::{Foo, Baz};
use ops::Add;

pub struct Bar<T> {}

impl<T> Foo for Bar<T> {
    /// something more about foo();
    fn foo() {}
}
impl<T> Baz for Bar<T> {}
impl<T> Bar<T> {
    fn foo_bar() {
        Self::foo()
    }
}

// test dependency impls
impl<T> Add for Bar<T> {
    fn add(self, other: Self) -> Self {
        Bar {}
    }
}
impl<T> ops::Subtract for Bar<T> {
    fn subtract(self, other: Self) -> Self {
        Bar {}
    }
}