script;
// This tests importing other files.

use foo::Foo;

fn main() -> bool {
  let foo = Foo {
    foo: "foo",
  };

  false
}

dep a_dependency;

