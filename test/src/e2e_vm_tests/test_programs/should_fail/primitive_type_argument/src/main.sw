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
  let _b = foo::bar::ExampleStruct::<bool> { a_field: 5u64 };
  return a.f();
}
