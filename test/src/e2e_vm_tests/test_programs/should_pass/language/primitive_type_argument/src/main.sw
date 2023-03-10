script; 

mod foo;

struct S<T> { }

impl<T> S<T> {
  fn f(self) -> u64 {
    5
  }
}

fn main() -> u64 {
  let a = S::<u64> { };
  let b = foo::bar::baz::ExampleStruct::<u64, bool> { a_field: 5u64, b_field: true };
  use foo::bar::baz::ExampleStruct;
  let c = foo::bar::baz::quux::Quux::<u64, bool, ExampleStruct<u64, bool>, u64, str[3], u64> { a: 10, b: true, c: b, d: 10, e: "foo", f: 10 };
  return a.f();
}
