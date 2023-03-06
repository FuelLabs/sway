script;

mod foo;
mod bar;
mod baz;

fn main() -> u64 {
    let x = foo::Foo {
        foo: 1u32,
    };
    let y = bar::Bar::Baz(true);
    let z = ::bar::Bar::Baz(false);
    baz::return_1()
}
