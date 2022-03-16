script;

use std::chain::*;
enum Result<T, E> {
  Ok: T,
  Err: E,
  }

fn local_panic() {
  asm() {
    rvrt zero;
  }
}

fn main() {
  // all of these should be okay, since 
  // the branches that would have type errors abort control flow.
  let x = if true { 42u64 } else { panic(0) };
  let x: u64 = local_panic();
  let x = if let Result::Ok(ok) = Result::Ok(5) { ok } else { local_panic() };
  let x = if true { 42u64 } else { return; };
}
