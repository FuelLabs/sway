script; 

struct S<T> { }

impl<T> S<T> {
  fn f(self) -> u64 {
    size_of::<T>()
  }
}

fn main() -> u64 {
  let a = S::<u64> { };
  return a.f();
}
