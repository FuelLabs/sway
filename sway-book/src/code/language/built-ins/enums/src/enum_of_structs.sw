library enum_of_structs;

// ANCHOR: content
struct Item {
    price: u64,
    amount: u64,
    id: u64,
}

enum MyEnum {
    Product: Item,
}

fn main() {
    let my_enum = MyEnum::Product(Item {
        price: 5,
        amount: 2,
        id: 42,
    });
}
// ANCHOR_END: content
