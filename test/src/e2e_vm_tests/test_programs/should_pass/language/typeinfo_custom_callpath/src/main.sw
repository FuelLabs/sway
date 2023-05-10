script;

mod foo;

struct Bar {
  baz: foo::Foo
}


fn main() {
    let x = Bar {
        baz: foo::Foo::A
    };

    let b = match x {
        Bar { baz: foo::Foo::A(_) } => true,
        _ => false,
    };

    assert(b);
}
