script;
// this file tests struct field reassignments
fn main() -> u64 {
  let mut data = Data { 
		uselessnumber: 42
    };

  return data.uselessnumber;
}

struct Data {
  uselessnumber: u64
}

