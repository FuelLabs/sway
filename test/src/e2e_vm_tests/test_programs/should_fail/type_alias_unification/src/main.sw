script;

trait MyTrait {
    fn extract_a(self) -> u64;
}

struct A {
    a: u64,
}

impl MyTrait for A {
    fn extract_a(self) -> u64 {
        self.a
    }
}

type B = A;

// B is an alias for A, and A already has an implementation of MyTrait,
// so this should cause a compilation error.
impl MyTrait for B {
    fn extract_a(self) -> u64 {
        self.a + 1
    }
}

fn main() {
    let struct_a = A { a: 1 }; 
    let struct_b = B { a: 42 };
    assert(struct_a.extract_a() == 1);
    assert(struct_b.extract_a() == 42);
}
