script;

struct S {
    x: u64,
}

fn main() {
    let thing: S = S { x: 0 };
    thing.x = 23;
}

