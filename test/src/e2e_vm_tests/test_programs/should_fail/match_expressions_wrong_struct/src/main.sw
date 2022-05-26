script;

enum Wow {
}

enum Foo {
    Bar: u32,
    Zoom: Wow,
}

struct Point {
    x: u64,
    y: u64
}

struct Data<T> {
    value: T
}

fn main() -> u64 {
    let a = 6;
    let b = match a {
        Point { x: 3, y } => { y },
        Point { x: 3, y: 4 } => { 24 },
        _ => { 24 },
    };

    let c = Data {
        value: true
    };
    let e = match c {
        Data { value: 1u64 } => { false },
        Data { value } => { true },
    };

    let f = Foo::Bar(30);
    let g = match f {
        Bar(x) => x,
    };

    0
}
