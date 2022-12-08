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
    // Set `foo` & `bar` to take the value of `5` and the default `u64` type
    let foo = 5;
    const bar = 5;

    // Reassign `foo` & `bar` to be a `str[4]` with the value of `Fuel`
    let foo = "Fuel";
    const bar = "Fuel";
    // ANCHOR_END: reassignment
}

fn shadowing() {
    // ANCHOR: shadowing
    let foo = 5;
    const bar = 5;
    {
        let foo = 42;
        const bar = 42;
    }
    assert(foo == 5);
    assert(bar == 5);
    // ANCHOR_END: shadowing
}

fn constants() {
    // ANCHOR: constants
    const FOO = 5;
    // ANCHOR_END: constants
}
