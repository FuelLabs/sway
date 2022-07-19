script;

struct Data<T> {
    value: T
}

impl<T> Data<T> {
  fn new(value: T) -> Data<T> {
    // this ↓↓↓↓↓   
    Data {
        val // val is not a data member so this should be a compile error
    }
  }
}

fn main() -> bool {
    false
}
