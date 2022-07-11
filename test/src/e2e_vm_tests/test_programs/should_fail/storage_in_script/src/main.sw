script;

// Expecting the following error:
//  9 | / storage {
// 10 | |     item: u64 = 0,
// 11 | | }
//    | |_^ Declaring storage in a script is not allowed.

storage {
    item: u64 = 0,
}

fn main() -> bool {
    false
}
