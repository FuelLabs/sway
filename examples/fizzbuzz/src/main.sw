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
    pub fn bar(&self, i: u8) -> u8 {
        i
    }
    pub fn baz(self) -> u8 {
        99
    }
    pub fn foo(&self, j: Foo) -> Foo {
        j
    }
}

fn main() {
    let fooo = Foo::new();
    fooo.foo(fooo).
}

fn otherfn(fooo: Foo) {
    let x = fooo.something.a;
}
