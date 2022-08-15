script;

struct Foo {
    value: u64
}

fn mut_foo(ref mut foo: Foo) {
    foo = Foo { value: 10 };
}

fn main() -> u64 {
    let mut foo = Foo { value: 0 };
    mut_foo(foo);
    foo.value
}
