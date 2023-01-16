script;

use core::ops::*;
use std::assert::*;

enum Result2<T, E> {
    Ok: T,
    Err: E,
}

impl<T, E> Result2<T, E> {
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Result2::Ok(inner_value) => inner_value,
            Result2::Err(_) => default,
        }
    }
}

pub fn test_unwrap_or<T>(val: T, default: T)
where
    T: Eq
{
    assert(Result2::Ok::<T, T>(val).unwrap_or(default) == val);
    assert(Result2::Err::<T, T>(val).unwrap_or(default) == default);
}

fn main() -> bool {
  test_unwrap_or(true, true);
  test_unwrap_or(true, false);


  true
}
