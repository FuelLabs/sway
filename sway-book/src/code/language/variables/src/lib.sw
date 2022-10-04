library variables;

fn mutable() {
    // ANCHOR: mutable
    let mut foo = 5;
    foo = 6;
    // ANCHOR_END: mutable
}

fn immutable() {
    // ANCHOR: immutable
    let foo = 5;
    // ANCHOR_END: immutable
}

fn reassignment() {
    // ANCHOR: reassignment
    let foo = 5;
    let foo = "Fuel";
    // ANCHOR_END: reassignment
}

fn shadowing() {
    // ANCHOR: shadowing
    let foo = 5;
    {
        let foo = 42;
    }
    assert(foo == 5);
    // ANCHOR_END: shadowing
}
