script;

fn return_value(data: Data) -> u64 {
  data.value()
}

// this file tests struct field reassignments
fn main() -> u64 {
  let data = Data { 
		uselessnumber: 42
  };

  let nested_data = NestedData {
    data: Data {
      uselessnumber: 99
    }
  };

  data.value() + return_value(data) + double(data)
}

impl Data {
  fn value(self) -> u64 {
    self.uselessnumber
  }
}

fn double(data: Data) -> Data {
  Data {
    uselessnumber: data.uselessnumber * 2
  }
}

struct Data {
  uselessnumber: u64
}

struct NestedData {
  data: Data
}