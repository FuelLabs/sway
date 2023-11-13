script;

fn main() -> u64 {
    let a = A { a: 11 };
    let mut ptr_a = ptr(a);
    ptr_a.write(A { a: 22 });
    assert(a.a == 11);
    a.a
}

struct A {
  a: u64,
}

#[inline(never)]
fn ptr<T>(t: T) -> raw_ptr {
    __addr_of(t)
}
