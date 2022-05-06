script;

enum X {
    Y: u64,
}

enum Sale {
    Cash: u64,
    Card: u64,
    Check: u64,
}

struct Point {
    x: u64,
    y: u64,
}

fn main() -> u64 {
    let a = X::Y(42);
    let b = match a {
        X::Y(hi) => { hi },
        _ => { 24 },
    };
    let c = match a {
        X::Y(10) => { 10 },
        _ => { 24 },
    };
    let d = Sale::Card(5);
    let e = match d {
        Sale::Check(_) => { 1 },
        Sale::Cash(_) => { 2 },
        Sale::Card(4) => { 3 },
        Sale::Card(_) => { 4 },
    };
    let f = Point {
        x: 0u64,
        y: 0u64
    };
    let g = match f {
        Point { x, y: 1 } => { 0 },
        Point { x: 1, y } => { 1 },
        Point { x, y } => { 2 },
    };

    if b == 42 && c == 24 && e == 4 && g == 2 {
        42
    } else {
        0
    }
}
