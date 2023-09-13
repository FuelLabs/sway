script;

enum Result<T, E> {
  Ok: T,
  Err: E,
}

fn foo(result_a: Result<u64, bool>, result_b: Result<u64, bool>) -> u64 {

  if let Result::Err(_a) = result_a {
    6
  } else if let Result::Ok(_num) = result_b {
    10
  } else if let Result::Ok(num) = result_a {
    num
  } else { 
    42 
  }
}

// should return 5
fn main() -> u64 {
  let result_a = Result::Ok::<u64, bool>(5u64);
  let result_b = Result::Err::<u64, bool>(false);
  foo(result_a, result_b)
}
