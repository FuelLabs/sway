script;

// should return 5
fn main() -> u64 {
  let result_a = Result::Ok::<u64, bool>(5u64);
  let result_b = Result::Err::<u64, bool>(false);

  if let Result::Err(a) = result_a {
    6
  } else if let Result::Err(some_bool) = result_b {
    10
  } else if let Result::Ok(num) = result_a {
    num
  } 
}
