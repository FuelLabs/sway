library;

// ANCHOR: trait_definition
trait Convert<T> {
    fn from(t: T) -> Self;
}
// ANCHOR_END: trait_definition

// ANCHOR: trait_impl
struct Square {
    width: u64,
}

struct Rectangle {
    width: u64,
    length: u64,
}

impl Convert<Square> for Rectangle {
    fn from(t: Square) -> Self {
        Self {
            width: t.width,
            length: t.width,
        }
    }
}
// ANCHOR_END: trait_impl

// ANCHOR: trait_usage
fn main() {
    let s = Square { width: 5 };
    let r = Rectangle::from(s);
}
// ANCHOR_END: trait_usage

// ANCHOR: trait_constraint
fn into_rectangle<T>(t: T) -> Rectangle
where
    Rectangle: Convert<T>
{
    Rectangle::from(t)
}
// ANCHOR_END: trait_constraint
