script;

enum Foo {
    Bar: Zoom,
}

enum Zoom {
    Wow: u32,
}

fn main() -> u32 {
    let x = Foo::Bar(Zoom::Wow(123));
    match x {
        Foo::Bar(Zoom::Wow(x)) => x,
    }
}
