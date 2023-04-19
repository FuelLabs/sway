script;


// Param `i` should have no warnings as `d` is still using it
fn unused_fn(i: u64) {
    let d = i;
}

fn f(i: u64) {
}

struct A {}

impl A {
    fn g(i: u64) {
    }

    fn h(self, i: u64) {
    }
}

fn i(_p: u64) {
}

fn j(ref mut foo: u64) {
    foo = 42;
}

fn main() {
    f(42);
    A::g(42);
    let a = A{};
    a.h(42);
    i(42);

    let mut foo = 42;
    j(foo);
}
