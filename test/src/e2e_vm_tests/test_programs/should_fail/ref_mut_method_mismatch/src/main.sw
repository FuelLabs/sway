script;

struct Point {
    x: u64,
    y: u64,
}

impl Point {
    fn mutate(self, ref mut _v: u64) { }
}

const MY_CONST: u64 = 10;

fn main() {
    // immutable variable
    let x = 42u64;
    Point { x: 1, y: 2 }.mutate(x);

    // struct field access on immutable binding
    let p = Point { x: 1, y: 2 };
    Point { x: 1, y: 2 }.mutate(p.x);

    // tuple element access on immutable binding
    let t = (1u64, 2u64);
    Point { x: 1, y: 2 }.mutate(t.0);

    // constant
    Point { x: 1, y: 2 }.mutate(MY_CONST);

    // mutable variable — should NOT error
    let mut m = 99u64;
    Point { x: 1, y: 2 }.mutate(m);
}
