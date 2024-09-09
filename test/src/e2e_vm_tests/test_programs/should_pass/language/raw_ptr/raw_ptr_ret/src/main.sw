script;

struct S {
    x: u64,
}

fn main() -> raw_ptr {
    __addr_of(S { x: 123 })
}
