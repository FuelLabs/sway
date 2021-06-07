script;
// This tests importing other files.

dep a_dependency;
dep nested_dependency/bar/bar;

use foo::Foo;
use ::foo::bar::Bar;

fn main() -> bool {
  let foo = Foo {
    foo: "foo",
  };
  return true;
}
