library;

// ANCHOR: content
struct Item {
    amount: u64,
    id: u64,
    price: u64,
}

enum MyEnum {
    Product: Item,
}

fn main() {
    let my_enum = MyEnum::Product(Item {
        amount: 2,
        id: 42,
        price: 5,
    });
}
// ANCHOR_END: content
