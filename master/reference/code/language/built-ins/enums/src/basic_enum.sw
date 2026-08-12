library;

// ANCHOR: definition
enum Color {
    Blue: (),
    Green: (),
    Red: (),
    Silver: (),
    Grey: (),
}
// ANCHOR_END: definition
fn main() {
    // ANCHOR: init
    // To instantiate an enum with a variant of the unit type the syntax is
    let blue = Color::Blue;
    let silver = Color::Silver;
    // ANCHOR_END: init
}
