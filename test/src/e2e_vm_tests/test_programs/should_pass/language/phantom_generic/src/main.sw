script;

struct RawVec<T> {
  ptr: u64,
  size: u64,
  len: u64,
}

impl<T> RawVec<T> {
  fn new() -> RawVec<T> {
    let size = size_of::<T>();
    RawVec {
      ptr: 0,
      size: size,
      len: 0,
    }
  }
}

fn main() {
  let x: RawVec<u64> = ~RawVec::new();

/*
  x.push(42);
  x.pop().unwrap()
*/
}
