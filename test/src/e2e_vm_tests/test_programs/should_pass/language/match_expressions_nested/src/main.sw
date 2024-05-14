script;

enum Foo {
    Bar: Zoom,
}

enum Zoom {
    Wow: u32,
}

fn match_me(me: Foo) -> u32 {
    match me {
        Foo::Bar(Zoom::Wow(11)) => 1111,
        Foo::Bar(Zoom::Wow(22) | Zoom::Wow(33) | Zoom::Wow(44)) => 223344,
        Foo::Bar(Zoom::Wow(x)) => x,
    }
}

use core::ops::{Eq, Add};

enum FooG<T>
    where T: Eq
{
    Bar: ZoomG<T>,
}

enum ZoomG<T>
    where T: Eq
{
    Wow: T,
}

fn match_generic<T>(me: FooG<T>) -> T
where T: Eq + Add {
    match me {
        FooG::Bar(ZoomG::Wow(x)) => x + x,
    }
}

fn main() -> u32 {
    let x = match_me(Foo::Bar(Zoom::Wow(11)));
    assert(x == 1111);

    let x = match_me(Foo::Bar(Zoom::Wow(22)));
    assert(x == 223344);

    let x = match_me(Foo::Bar(Zoom::Wow(33)));
    assert(x == 223344);

    let x = match_me(Foo::Bar(Zoom::Wow(44)));
    assert(x == 223344);

    let x = match_me(Foo::Bar(Zoom::Wow(1234)));
    assert(x == 1234);

    let x: u8 = match_generic(FooG::Bar(ZoomG::Wow(21u8)));
    assert(x == 42u8);

    let x = match_generic(FooG::Bar(ZoomG::Wow(21u8)));
    assert(x == 42u8);

    let x = match_generic(FooG::Bar(ZoomG::Wow(21u32)));
    assert(x == 42u32);

    let x = match_generic(FooG::Bar(ZoomG::Wow(21)));
    assert(x == 42);

    123
}
