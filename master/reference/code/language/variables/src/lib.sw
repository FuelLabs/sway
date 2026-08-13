library;

fn mutable() {
    // ANCHOR: mutable
    let mut foo = 5;
    foo = 6;
    // ANCHOR_END: mutable
}

fn mutable_evaluation_order() {
    // ANCHOR: mutable_evaluation_order
    let mut array = [0, 0, 0];
    let mut i = 0;

    array[i] = {
        i += 1;
        i
    };
    // ANCHOR_END: mutable_evaluation_order
}

fn immutable() {
    // ANCHOR: immutable
    let foo = 5;
    // ANCHOR_END: immutable
}

fn reassignment() {
    // ANCHOR: reassignment
    // Set `foo` to take the value of `5` and the default `u64` type
    let foo = 5;

    // Reassign `foo` to be a `str` with the value of `Fuel`
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

fn constants() {
    // ANCHOR: constants
    const FOO = 5;
    // ANCHOR_END: constants
}
