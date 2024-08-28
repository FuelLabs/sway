script;

struct S {}

impl S {
    fn one_generic<T>(self) { }
    fn two_generics<A, B>(self) { }
}

struct W<A> { }

fn main() {
    let g: bool = three_generics(true, "foo", 10);

    // Should fail because compiler cannot infer generic argument T
    one_generic();

    // Should fail because compiler cannot infer generic arguments A, B
    two_generics();

    // Two generics arguments expected
    two_generics::<u64>();

    // Should fail because compiler cannot infer generic argument T
    S{}.one_generic();

    // Should fail because compiler cannot infer generic arguments A, B
    S{}.two_generics();

    // Two generics arguments expected
    S{}.two_generics::<u64>();

    // Missing generic argument of W
    one_generic::<W>();
}

fn three_generics(a: A, b: B, c: C) -> A {
    let new_a: A = a;
    new_a
}

fn one_generic<T>() { }
fn two_generics<A, B>() { }
