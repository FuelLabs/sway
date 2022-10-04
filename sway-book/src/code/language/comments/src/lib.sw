library comments;

fn single_line() {}

    // ANCHOR: single_line
    // hello world
    // ANCHOR_END: single_line
fn multi_line() {}

    // ANCHOR: multi_line
    // imagine that this line is twice as long
    // and it needed to be split onto multiple lines
    // ANCHOR_END: multi_line
fn end_of_line() {
    // ANCHOR: end_of_line
    let baz = 8;  // Eight is a good number
    // ANCHOR_END: end_of_line
}

fn block() {
    // ANCHOR: block
    /*
        You can write on multiple lines
        like this if you want
    */
    let baz = 8;
    // ANCHOR_END: block
}
