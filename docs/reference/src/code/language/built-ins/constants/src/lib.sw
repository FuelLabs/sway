library;

struct S {}

impl S {
    const ID: u64 = 0;
}

// ANCHOR: id
fn returns_id() -> u64 {
    S::ID
}
// ANCHOR_END: id
