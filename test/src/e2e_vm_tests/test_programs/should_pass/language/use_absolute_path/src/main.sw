script;

mod r#trait;
mod foo;

use ::foo::*;
use ::trait::*;
use std::assert::*;

struct S<T> where T: Trait {}

fn main() -> u64 {
    assert(Foo::method() == 42);

    let _s = S::<Foo> {};

    1
}
