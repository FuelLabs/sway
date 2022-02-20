script;

struct Data {
  one: u64,
  two: u64,
  three: u64,
  four: u64,
  five: u64
}

fn main() -> u64 {
    let x = Data {
        one: 1,
        two: 2,
        three: 3,
        four: 4,
        five: 5,
    };
    size_of_val(x)
}
