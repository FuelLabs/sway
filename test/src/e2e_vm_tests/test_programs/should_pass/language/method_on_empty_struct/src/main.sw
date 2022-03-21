script;

struct A { }

impl A {
    fn f(self) -> u64 {
        1
    }
}

fn main() -> u64 {
    let a = A { };
    a.f()
}
