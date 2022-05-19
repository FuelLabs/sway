script;

struct S { }

impl<T> S {
  fn f(self, value: T) {
    __size_of::<T>();
  }
}

fn main() {
    (S{}).f(true);
}
