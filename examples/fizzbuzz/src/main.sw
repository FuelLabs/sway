script;

struct Something {
    a: u32,
    b: Something2,
}

struct Something2 {
    c: u32,
}

struct Foo {
    lol: u32,
    something: Something,
}

impl Foo {
    pub fn new() -> Self {
        Self {
            lol: 1,
            something: Something {
                a: 2,
                b: Something2 { c: 3 },
            },
        }
    }
    pub fn bar(i: u8) -> u8 {
        i
    }
    pub fn baz() -> u8 {
        99
    }
    pub fn foo(j: Foo) -> Foo {
        j
    }
}

fn main() {
    let fooo = Foo::new();
    fooo
}

fn otherfn(fooo: Foo) {
    let x = fooo.something.a;
}
