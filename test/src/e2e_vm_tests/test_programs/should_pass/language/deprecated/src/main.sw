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
}