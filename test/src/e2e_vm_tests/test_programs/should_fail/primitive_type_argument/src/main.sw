script; 

dep foo;

struct S<T> { }

impl<T> S<T> {
  fn f(self) -> u64 {
    5
  }
}

fn main() -> u64 {
  let a = S::<u64> { };
  let b = foo::baz::ExampleStruct::<bool> { a_field: 5u64 };
  return a.f();
}
