script;

struct Struct {
}

impl Struct {
    pub fn foo(self) -> u32 { 10 }
    pub fn bar(self) -> u32 { self.foo() }
}

fn main() -> u32 {
    let s = Struct {};
    s.bar()
}
