script;

mod foo;
mod bar;
mod baz;

fn main() -> u64 {
    let _x = foo::Foo {
        foo: 1u32,
    };
    let _y = bar::Bar::Baz(true);
    let _z = ::bar::Bar::Baz(false);
    baz::return_1()
}
