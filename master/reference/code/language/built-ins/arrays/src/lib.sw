library;

// ANCHOR: syntax
fn syntax() {
    let array = [1, 2, 3, 4, 5];

    let mut counter = 0;
    let mut total = 0;

    while counter < 5 {
        total += array[counter];
        counter += 1;
    }
}
// ANCHOR_END: syntax
