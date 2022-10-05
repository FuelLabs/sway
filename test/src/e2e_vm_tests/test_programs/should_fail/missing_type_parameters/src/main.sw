script;

fn main() {
    let g: bool = three_generics(true, "foo", 10);
}

fn three_generics(a: A, b: B, c: C) -> A {
    let new_a: A = a;
    new_a
}
