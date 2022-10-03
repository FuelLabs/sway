library basic_enum;

// ANCHOR: content
// Declare the enum
enum Color {
    Blue: (),
    Green: (),
    Red: (),
    Silver: (),
    Grey: (),
}

fn main() {
    // To instantiate a variable with the value of an enum the syntax is
    let blue = Color::Blue;
    let silver = Color::Silver;
}
// ANCHOR_END: content
