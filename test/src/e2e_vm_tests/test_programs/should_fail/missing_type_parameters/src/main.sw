script;

fn main() {
    let g: bool = three_generics(true, "foo", 10);
    
    // Should fail because compiler cannot infer generic argument T
    one_generic();

    // Should fail because compiler cannot infer generic arguments A, B
    two_generics();

    // Two generics arguments expected
    two_generics::<u64>();
}

fn three_generics(a: A, b: B, c: C) -> A {
    let new_a: A = a;
    new_a
}

fn one_generic<T>() { }
fn two_generics<A, B>() { }
