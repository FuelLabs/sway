// This should result in an error saying that the method signature of the
// implementation does not match the declaration.

library test;

struct MyStruct<T> {
    val: T
}

trait MyTrait<T> {
    fn set(self, val: T);
}

impl<T> MyTrait<T> for MyStruct<T> {
    // This implementation uses an Option, but the definition does not
    fn set(self, val: Option<T>) {

    }
}

trait Foo {
    fn foo(x: u64) -> str[7];
    fn bar(variable: u64) -> bool;
    fn baz() -> u32;
    fn quux() -> u64;
}

struct S {
    x: u64,
}

impl Foo for S {
    fn foo(s: str[7]) -> str[7] {
        s
    }

    fn bar(ref mut variable: u64) -> bool {
        true
    }

    fn baz() -> u64 {
        0
    }

    fn quux() { // no return type
    }
}
