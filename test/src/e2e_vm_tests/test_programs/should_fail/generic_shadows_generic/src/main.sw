script;

struct Data<T> {
  value: T
}

impl<T> Data<T> {
  fn new(value: T) -> Data<T> {
    Data {
        value: value
    }
  }

  fn get_value<T>(self) -> T {
    self.value
  }
}

fn main() -> bool {
    let data = Data::new(7);
    let foo = data.get_value();
    true
}
