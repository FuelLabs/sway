script;

enum Result<T, E> {
  Ok: T,
  Err: E,
}

// should return 5
fn main() -> u64 {
  let result_a = Result::Ok(5u64);
  let result_b = Result::Err(false);

  if let Result::Err(a) = result_a {
    6
  } else if let Result::Err(some_bool) = result_b {
    10
  } else if let Result::Ok(num) = result_a {
    num
  } else { 42 }
}
