script;

dep a_dependency;

fn main() -> u64 {
    let foo = foo::Foo {
        foo: 1u32,
    };
    foo.foo
}
