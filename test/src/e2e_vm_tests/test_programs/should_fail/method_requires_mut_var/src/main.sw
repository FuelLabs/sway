script;

struct A {
    a: u64,
}

impl A {
    fn f(ref mut self) {
        self.a = 42;
    }
}

fn main() -> bool {
    let a = A {
        a: 0,
    };

    // Expecting error: Cannot call method "f" on variable "a" because "a" is not declared as mutable.
    a.f();
    
    false
}
