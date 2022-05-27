script;
use std::{result::*, revert::*, u128::*};

fn foo(a: u64, b: u64, c: u64) -> u64 {
    let result_wrapped: Result<u64, ()> = ((~U128::from(0, a) * ~U128::from(0, b)) / ~U128::from(0, c)).to_u64();
    let result = result_wrapped.unwrap();
    let other_result: Result<u64, u64> = Result::<u64, u64>::Err(5);
    other_result.unwrap();
    result
}

fn main() -> u64 {
    let a = foo(1, 2, 2);
    let b = foo(5, 5, 5);
    a + b - 1
}
