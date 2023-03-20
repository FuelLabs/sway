script;
// This tests importing other files.

dep a_dependency;
dep nested_dependency/bar/bar;

use foo::Foo;
use ::foo::bar::{Bar, double_bar::DoubleBar};

fn main() -> bool {
    let _foo = Foo {
        foo: "foo",
    };
    let _db = ::foo::bar::double_bar::DoubleBar {
        a: 5u32,
    };
    let _bar = Bar {
        a: 5u32,
    };
    false
}
