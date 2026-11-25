library;

#[allow(dead_code)]
enum Shape {
    Triangle: (),
    Quadrilateral: (),
    Pentagon: (),
    Hexagon: (),
    Heptagon: (),
}

#[allow(dead_code)]
// ANCHOR: style_match_unnamed
fn unnamed_case(shape: Shape) {
    let value = match shape {
        Shape::Triangle => 3,
        Shape::Quadrilateral => 4,
        Shape::Pentagon => 5,
        _ => 0,
    };
}
// ANCHOR_END: style_match_unnamed

#[allow(dead_code)]
// ANCHOR: style_match_named
fn named_case(shape: Shape) {
    let value = match shape {
        Shape::Triangle => 3,
        Shape::Quadrilateral => 4,
        Shape::Pentagon => 5,
        _invalid_shape => 0,
    };
}
// ANCHOR_END: style_match_named
