script;

struct A {
    a: u64,
}

impl A {
    fn f(self) {
        self.a = 0;
    }
}

fn main() {
    let a = A { a: 0 };
    a.f();
}
