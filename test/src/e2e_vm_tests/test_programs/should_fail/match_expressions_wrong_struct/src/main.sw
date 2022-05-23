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

fn main() -> u64 {
    let p = 6;
    let g = Foo::Bar(30);
    let h = match g {
        Bar(x) => x,
    };

    match p {
        Point { x: 3, y } => { y },
        Point { x: 3, y: 4 } => { 24 },
        _ => { 24 },
    }
}
