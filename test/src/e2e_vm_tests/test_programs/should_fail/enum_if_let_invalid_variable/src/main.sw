script;

enum Result<T, E> {
  Ok: T,
  Err: E,
}

fn main() -> u64 {
   let x: Result<u64, u64> = Result::Ok::<u64, u64>(5u64);

   // this should fail because "y" is not available in the scope of the "else"
   if let Result::Ok(y) = x { y + 10 } else { y }
}
