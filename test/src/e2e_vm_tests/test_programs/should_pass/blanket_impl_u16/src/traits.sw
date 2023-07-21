library;

pub trait Foo {
    fn foo(self) -> u64;
}

impl<T> Foo for T {
    fn foo(self) -> u64 {
        42
    }
}