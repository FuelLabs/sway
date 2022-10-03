library arrays;

// ANCHOR: syntax
fn syntax() {
    // Annotation not required
    let array: [u64; 5] = [1, 2, 3, 4, 5];

    let mut counter = 0;
    let mut total = 0;

    while counter < 5 {
        total += array[counter];
        counter += 1;
    }

    // Not currently supported
    // array[1] = 42;
}
// ANCHOR_END: syntax
