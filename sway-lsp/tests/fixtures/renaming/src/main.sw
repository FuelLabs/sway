script;

enum Color {
    Red: (),
    Green: (),
    Blue: (),
}

struct Point {
    x: u32,
    y: u32,
}

fn add(x: u32, y: u32) -> u32 {
    x + y
}

fn main() {
    let c = Color::Red;
    let point = Point { x: 10, y: 20 };
    let n = add(point.x, point.y);
    let f = (c, point, n);
    // raw identifier syntax 
    let r#struct = ();
    let d = r#struct;
}
