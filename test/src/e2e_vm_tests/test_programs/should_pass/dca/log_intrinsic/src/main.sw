script;

fn encode<T>(_item: T) -> raw_slice {
    asm(ptr: (0, 0)) { ptr: raw_slice }
}

struct Foo {
    value: u64
}

fn main() -> u64 {
    __log(Foo {value: 0});
    0
}
