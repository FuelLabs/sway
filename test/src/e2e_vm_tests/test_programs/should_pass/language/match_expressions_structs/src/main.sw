script;

struct Point {
    x: u64,
    y: u64
}

struct Data<T> {
    value: T
}

fn main() -> u64 {
    let a = Point {
        x: 3,
        y: 4,
    };
    let b = match a {
        Point { x: 3, y } => { y },
        Point { x: 3, y: 4 } => { 24 },
        _ => { 24 },
    };

    let c = Data {
        value: true
    };
    let d = match c {
        Data { value: false } => { 0 },
        Data { value } => { 4 },
    };

    d
}
