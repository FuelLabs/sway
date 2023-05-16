script;

struct A {
    a: u64,
}

struct B {
    a: A,
}

struct C {
    b: B,
}

impl A {
    fn f(ref mut self) {
        self.a = 42;
    }
}

impl B {
    fn f(self) {
        // Expecting error: Cannot call method "f" on variable "self" because "self" is not declared as mutable.
        self.a.f();
    }
}

impl C {
    fn f(self) {
        // Expecting error: Cannot call method "f" on variable "self" because "self" is not declared as mutable.
        self.b.a.f();
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
