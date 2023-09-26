library;

#[deprecated]
struct A {
}

#[deprecated]
enum B {
    A: ()
}

pub fn f() {
    let _ = A {};
    let _ = B::A;

    use std::u256::U256;
    let _ = U256::new();
}