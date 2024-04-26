library;

use ::bar::Bar as Baz; // This is reexported at Baz

pub struct Foo {
    pub foo: u64,
}
