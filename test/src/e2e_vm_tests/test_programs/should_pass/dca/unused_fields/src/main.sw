script;

dep utils;
use utils::Foo;


struct Bar {
    value: u64
}

fn internal_fn(s: Bar) {

}

fn main() -> u64 {
    internal_fn(Bar {value: 0});
    utils::external_fn(Foo {value: 0});
    0
}
