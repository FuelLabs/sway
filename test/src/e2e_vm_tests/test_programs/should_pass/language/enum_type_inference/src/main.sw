script;

// "Large Type" here means larger than one word. Different assembly is used to destructure inner types of > 1 word.
// This file tests the correctness of that.

enum Result<T, E> {
  Ok: T,
  Err: E,
}

struct Product {
}

struct ItemDetails {
}

enum SaleError {
    NotEnoughInventory: str[3], 
}

fn main() -> u64 {
    let x = Result::Ok::<u64, SaleError>(5u64);
    let mut _y = Result::Err::<u64, SaleError>(SaleError::NotEnoughInventory("foo"));
    // should be the same type
    _y = x;
    5
}

