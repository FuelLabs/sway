script; 

struct S<T> { }

impl<T> S<T> {
  fn f(self) -> u64 {
    size_of::<T>()
  }
}

fn main() -> u64 {
  let a = S::<u64> { };
  // the `<T>` on line 5 should be known, since we associated it with the type above.
  return a.f();
}
