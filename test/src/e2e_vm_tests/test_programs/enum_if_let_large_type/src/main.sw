script;

// "Large Type" here means larger than one word. Different assembly is used to destructure inner types of > 1 word.
// This file tests the correctness of that.

enum Result<T, E> {
  Ok: T,
  Err: E,
}

struct Product {
  details: ItemDetails,
  inventory_number: u64,
  number_sold: u64,
  number_available: u64,
}

struct ItemDetails {
  name: str[4],
  price: u64,
}

enum SaleError {
    NotEnoughInventory: str[3], 
}

fn main() -> u64 {
    let x = Result::Ok(5u64);

    // should return 15
    if let Result::Ok(y) = x { y + 10 } else { 1 }
}

fn sell_product(product: Product) -> Result<Product, SaleError> {
  let mut product = product;
  if product.number_available < 1 {
    return Result::Err(SaleError::NotEnoughInventory("noo"));
  };
  product.number_sold = product.number_sold + 1;
  product.number_available = product.number_available - 1; 
  return Result::Ok(product);
// why is this unresolved?
// return Result::Ok::<Product, SaleError>(product);
}
