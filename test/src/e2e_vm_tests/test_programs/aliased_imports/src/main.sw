script;
// This tests importing other files.

dep a_dependency;

use foo::Foo as MyFoo;

fn main() -> u64 {
    let foo = MyFoo {
        foo: 42,
    };
    foo.foo
}
