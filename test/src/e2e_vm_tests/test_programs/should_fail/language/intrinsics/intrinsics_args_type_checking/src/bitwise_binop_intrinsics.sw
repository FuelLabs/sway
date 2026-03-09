library;

struct A {
    a: u64,
}

enum B {
  First: (),
  Second: u64
}

pub fn check_args() {
    let _ = __rsh();
    let _ = __rsh(42u64);
    let _ = __rsh((), 42u64);
    let _ = __rsh(42u64, 1u32);
    let _ = __rsh::<u64>(42u64, 1u64);
    let _ = __rsh::<u32>(42, 1);

    let _ = __eq("hi", "ho");
    let _ = __eq(false, 11);
    let _ = __eq(A { a: 1 }, B { a: 1 });
    let _ = __eq(A { a: 1 }, A { a: 1 });
    let _ = __eq((1, 2), (1, 2));
    let _ = __eq([1, 2], [1, 2]);
    let _ = __eq(B::First, B::First);
    let _ = __eq(B::Second(1), B::Second(1));
}
