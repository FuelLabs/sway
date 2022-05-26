script;

struct Vec<T> {
  ptr: u64
}

impl<T> Vec<T> { 
  fn foo(self) -> u64 {
    let size = __size_of::<T>();
    size
  }
  fn new() -> Self {
    Vec::<T> {
      ptr: 0
    }
  }
}

fn main() -> u64 {
  let z: Vec<u64> = Vec::new();
  z.foo()
}
