script;

dep foo;

use foo::*;

struct Bar {
  baz1: foo::Foo,
  baz2: Foo,
}

fn main() {
    let x = Bar {
        baz1: Foo::A,
        baz2: foo::Foo::A,
    };

    let b = match x {
        Bar { baz1: ::foo::Foo::A(_), baz2: Foo::A(_) } => true,
        _ => false,
    };

    assert(b);
}
