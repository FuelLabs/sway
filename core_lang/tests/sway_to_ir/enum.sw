script;

enum Fruit {
    Apple: (),
    Banana: (),
    Grapes: u64,
}

fn main() {
    let lunch = Fruit::Banana;
    eat(lunch);
    eat(Fruit::Grapes(3));
}

fn eat(meal: Fruit) -> bool {
    false
}
