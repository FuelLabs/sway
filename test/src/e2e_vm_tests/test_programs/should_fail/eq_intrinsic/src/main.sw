script;

struct A {
    a: u64,
}

enum B {
  First: (),
  Second: u64
}

fn main() {
    let _ = __eq("hi", "ho");
    let _ = __eq(false, 11);
    let _ = __eq(A { a: 1 }, B { a: 1 });
    let _ = __eq((1, 2), (1, 2));
    let _ = __eq([1, 2], [1, 2]);
    let _ = __eq(B::First, B::First);
    let _ = __eq(B::Second(1), B::Second(1));
    let my_number1: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let my_number2: b256 = 0x000000000000000000000000000000000000000000000000000000000000002A;
    let _ = __eq(my_number1, my_number1);
}
