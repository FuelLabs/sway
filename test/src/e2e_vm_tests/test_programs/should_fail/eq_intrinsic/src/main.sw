library;

struct A {
    a: u64,
}

enum B {
  First: (),
  Second: u64
}

pub fn main() {
    let _ = __eq("hi", "ho");
    let _ = __eq(false, 11);
    let _ = __eq(A { a: 1 }, B { a: 1 });
    let _ = __eq(A { a: 1 }, A { a: 1 });
    let _ = __eq((1, 2), (1, 2));
    let _ = __eq([1, 2], [1, 2]);
    let _ = __eq(B::First, B::First);
    let _ = __eq(B::Second(1), B::Second(1));
}
