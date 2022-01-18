script;

struct S {
    a: bool,
    b: bool,
}

fn new(a: bool, b: bool) -> S {
    let a = false;
    let b = true;
    S {
        a: a,   // These should be the locals, not the args which are shadowed.
        b: b,
    }
}

fn main() {
    new(true, false);
}
