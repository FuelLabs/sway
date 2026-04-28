library;

// ANCHOR: struct_shorthand_definition
struct Structure {
    amount: u64,
}
// ANCHOR_END: struct_shorthand_definition
// ANCHOR: struct_shorthand_use
fn call(amount: u64) {
    let structure = Structure { amount };
}
// ANCHOR_END: struct_shorthand_use
// ANCHOR: struct_shorthand_avoid
fn action(value: u64) {
    let amount = value;
    let structure = Structure { amount: value };
    let structure = Structure { amount: amount };
}
// ANCHOR_END: struct_shorthand_avoid
