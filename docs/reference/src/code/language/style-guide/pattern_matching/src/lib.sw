library;

#[allow(dead_code)]
// ANCHOR: style_match_unnamed
fn unnamed_case(variable: u64) {
    let value = match variable {
        1 => 1,
        2 => 3,
        3 => 5,
        _ => 0,
    };
}
// ANCHOR_END: style_match_unnamed

#[allow(dead_code)]
// ANCHOR: style_match_named
fn named_case(variable: u64) {
    let value = match variable {
        1 => 1,
        2 => 3,
        3 => 5,
        default => 0,
    };
}
// ANCHOR_END: style_match_named
