script;

dep a_dependency;
dep b_dependency;

fn main() -> u64 {
    let x = foo::Foo {
        foo: 1u32,
    };
    let y = bar::Bar::Baz(true);
    x.foo
}
