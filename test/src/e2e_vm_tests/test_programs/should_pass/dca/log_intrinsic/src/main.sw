script;

struct Foo {
    value: u64
}

fn main() -> u64 {
    __log(Foo {value: 0});
    0
}
