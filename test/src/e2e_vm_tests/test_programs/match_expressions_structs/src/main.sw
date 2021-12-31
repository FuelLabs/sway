script;

struct Point {
    x: u64,
    y: u64
}

fn main() -> u64 {
    let p = Point {
        x: 3,
        y: 4,
    };

    match p {
        Point { x: 3, y } => { y },
        Point { x: 3, y: 4 } => { 24 },
        _ => { 24 },
    }
}
