script;

dep foo;

struct Bar {
  baz: foo::foo::Foo
}

struct Bar2 {
  baz: foo::Foo
}

fn main() {
}
