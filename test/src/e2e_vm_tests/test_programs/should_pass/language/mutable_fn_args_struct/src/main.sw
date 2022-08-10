script;

struct Foo {
    value: u64
}

impl Foo {
    pub fn set(mut self, value: u64) -> u64 {
        self.value = value;
        self.value
    }
}

fn mut_foo(mut foo: Foo) {
    foo.set(10);
}

fn main() -> u64 {
    let mut foo = Foo { value: 0 };
    mut_foo(foo);
    foo.value
}