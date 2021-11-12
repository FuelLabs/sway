script;

struct Foo<T> {
  a: T,
}

fn main() -> bool {
  let foo = Foo { a: true };
  let bar = Foo { a: 10 };

  foo.a
}
