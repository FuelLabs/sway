script;

struct S {
    x: (u8, u8),
}

impl S {
    fn method(self) {}
}

fn main() {
    S { x: 0 } = 0;
    S { x: 0 }.x = 0;

    return_array() = 1;
    return_array()[0].x.1 = 1;

    let s = S { x: (0, 0) };

    s.method() = 2;
    s.method().x = 2;

    return = 3;
    
    break = 4;

    continue = 5;

    (2 + 2) = 6;
    (2 + 2).x = 6;

    2 + 2 = 4;

    { } = 7;
    { s }.x = 7;
}

fn return_array() -> [S;2] {
    let s = S { x: (0, 0) };
    [s, s]
}