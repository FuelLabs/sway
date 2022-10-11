library tuples;

// ANCHOR: syntax
fn syntax() {
    // Define a tuple containing 2 u64 types
    let mut balances = (42, 1337);

    // Index into the tuple to get the first and second values
    let first = balances.0;
    let second = balances.1;

    // You can interntally mutate a tuple value as long as the type is the same
    balances.0 = 12;

    // Will error since "true" is a boolean but the tuple expects a u64
    // balances.0 = true;
    // You can overwrite the entire tuple as long as the types are the same
    balances = (3, 4);

    // Destructure the values from the tuple into variables
    let (first, second) = balances;

    // You may ignore values using "_"
    let (_, second) = balances;
}
// ANCHOR_END: syntax
// ANCHOR: arity
fn arity() {
    // x is of type u64
    let x: u64 = (42);

    // y is of type u64
    let y: (u64) = (42);

    // z is of type (u64), i.e. a one-arity tuple
    let z: (u64, ) = (42, );

    // type error
    // let w: (u64) = (42,);
}
// ANCHOR_END: arity
