script;

struct Foo<T> {
  a: T,
}

fn get_a<V>(foo: Foo<V>) -> V {
  foo.a
}

fn main() -> bool {
  let foo = Foo { a: true };
  let bar = Foo { a: 10 };

  get_a(foo)
}
