library;

fn syntax() {
    // ANCHOR: declare
    // Define a tuple containing 2 u64 types
    let mut balances = (42, 1337);
    // ANCHOR_END: declare
    // ANCHOR: index
    // first = 42, second = 1337
    let first = balances.0;
    let second = balances.1;
    // ANCHOR_END: index
    // ANCHOR: internal_mutability
    // 12 has the same type as 42 (u64) therefore this is valid
    balances.0 = 12;

    // true is a Boolean and the tuple expects a u64 therefore this is invalid
    // balances.0 = true;
    // ANCHOR_END: internal_mutability
    // ANCHOR: mutability
    // 3 is the same type as 42 (u64) and so is 4 and 1337 therefore this is valid
    balances = (3, 4);
    // ANCHOR_END: mutability
    // ANCHOR: destructure
    // first = 42, second = 1337
    let (first, second) = balances;
    // ANCHOR_END: destructure
    // ANCHOR: ignore_destructure
    // 42 is ignored and cannot be used
    let (_, second) = balances;
    // ANCHOR_END: ignore_destructure
}
