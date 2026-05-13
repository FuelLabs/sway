library;

// ANCHOR: syntax
fn returns_true() -> bool {
    let is_true = true;
    let is_false = false;

    // implicitly returns the Boolean value of `true`
    is_true == !is_false
}
// ANCHOR_END: syntax
