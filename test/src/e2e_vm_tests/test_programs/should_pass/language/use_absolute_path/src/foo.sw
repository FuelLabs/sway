library;

use ::trait::*;

pub struct Foo {
    foo: u32,
}

impl Trait for Foo {
    fn method() -> u64 {
        42
    }
}
