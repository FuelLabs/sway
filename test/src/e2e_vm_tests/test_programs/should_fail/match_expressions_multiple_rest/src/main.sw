script;

struct Point {
  x: u64,
  y: u64
}

// this should fail because of multiple rest patterns 

fn main() -> u64 {
    let p = Point {
        x: 1u64,
        y: 2u64,
    };

    match p {
        Point { x, .., .. } => { x },
    };

    0
}
