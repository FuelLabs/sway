library;

// ANCHOR: u64_example
pub enum T {
    a: u64,
    b: (),
}
// ANCHOR_END: u64_example
fn u64_unit_space() {
    // ANCHOR: u64_unit_space
    let b = T::b;
    // ANCHOR_END: u64_unit_space
}

fn u64_type_space() {
    // ANCHOR: u64_type_space
    let a = T::a(42);
    // ANCHOR_END: u64_type_space
}

// ANCHOR: b256_example
pub enum K {
    a: b256,
    b: u64,
}
// ANCHOR_END: b256_example
fn b256_unit_space() {
    // ANCHOR: b256_unit_space
    let b = K::b(42);
    // ANCHOR_END: b256_unit_space
}

fn b256_type_space() {
    // ANCHOR: b256_type_space
    let a = K::a(0x0000000000000000000000000000000000000000000000000000000000000000);
    // ANCHOR_END: b256_type_space
}
