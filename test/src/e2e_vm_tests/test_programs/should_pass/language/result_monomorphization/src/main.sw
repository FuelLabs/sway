script;
use std::{result::*, revert::*, u128::*};

fn foo(a: u64, b: u64, c: u64) -> u64 {
    let result_wrapped: Result<u64, ()> = ((~U128::from(0, a) * ~U128::from(0, b)) / ~U128::from(0, c)).to_u64();
    let result = result_wrapped.unwrap();
    let is_ok = result_wrapped.is_ok();
    let result2: Result<u64, u64> = Result::Err(5);
    let is_ok = result2.is_ok();
    result
}

fn main() -> u64 {
    let a = foo(1, 2, 2);
    let b = foo(5, 5, 5);
    a + b - 1
}
