script;

struct Foo {
    bar: u64,
    baz: bool,
}

impl Foo {
    // this is a _method_, as it takes `self` as a parameter.
    fn is_baz_true(self) -> bool {
        self.baz
    }

    // this is an _associated function_, since it does not take `self` as a parameter.
    // it is at the same time a _constructor_ because it instantiates and returns
    // a new instance of `Foo`.
    fn new_foo(number: u64, boolean: bool) -> Foo {
        Foo {
            bar: number,
            baz: boolean,
        }
    }
}

fn main() {
    let foo = Foo::new_foo(42, true);
    assert(foo.is_baz_true());
}
