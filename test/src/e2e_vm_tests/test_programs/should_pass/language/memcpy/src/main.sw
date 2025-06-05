script;

fn main() -> u64 {
    let a = A { a: 11 };
    mutate_arg_via_ptr(a);
    assert(a.a == 11);

    let b = A { a: 22 };
    let ptr_b = __addr_of(b);
    let ptr_t = passthrough(ptr_b);
    ptr_t.write(A { a: 44 });
    assert(b.a == 44);

    a.a
}

struct A {
  a: u64,
}

#[inline(never)]
fn mutate_arg_via_ptr<T>(t: T) {
    let ptr_t = __addr_of(t);
    ptr_t.write(A { a: 33 });
}

#[inline(never)]
fn passthrough(ptr: raw_ptr) -> raw_ptr {
    ptr
}