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
    let x = sell_product(Product {
        details: ItemDetails {
            name: "shoe", price: 100, 
        },
        inventory_number: 0, number_sold: 10, number_available: 5
    });

    // should return 15
    if let Result::Ok(y) = x {
        y.number_sold + 4
    } else {
        1
    }
}

fn sell_product(product: Product) -> Result<Product, SaleError> {
    let mut product = product;
    if product.number_available < 1 {
        return Result::Err::<Product, SaleError>(SaleError::NotEnoughInventory("noo"));
    };
    product.number_sold = product.number_sold + 1;
    product.number_available = product.number_available - 1;
    return Result::Ok(product);
}
