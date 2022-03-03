script;

struct Data {
  uselessnumber: u64
}

fn return_value(data: Data) -> u64 {
  data.value()
}

// this file tests struct field reassignments
fn main() -> u64 {
  let data = Data { 
		uselessnumber: 42
  };

  data.value() + return_value(data)
}

impl Data {
  fn value(self) -> u64 {
    self.uselessnumber
  }
}