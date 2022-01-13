script;

fn main() -> bool {
    let record = Record {
        a: false,
        b: Fruit::Apple,
    };
    record.a
}

struct Record {
    a: bool,
    b: Fruit,
}

enum Fruit {
    Apple: (),
    Banana: (),
    Grapes: u64,
}

