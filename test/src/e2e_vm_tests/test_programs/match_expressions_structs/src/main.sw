script;

struct Point {
    x: u32,
    y: u32
}

fn main() -> u64 {
    let p = Point {
        x: 3,
        y: 4,
    };

    match p {
        Point { 3, y } => { y },
        Point { 3, 4 } => { 24 },
    }
}
