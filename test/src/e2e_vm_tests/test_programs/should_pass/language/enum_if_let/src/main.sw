script;

enum Result<T, E> {
  Ok: T,
  Err: E,
}

fn main() -> u64 {
   let x: Result<u64, u64> = Result::Ok::<u64, u64>(5u64);

   let result_1 = if let Result::Ok(x) = x { 100 } else { 1 };
   let result_2 = if let Result::Err(x) = x { 3 } else { 43 };
   result_1 + result_2
}
