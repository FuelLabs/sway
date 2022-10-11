contract;

struct A {
    field: bool,
}

fn foo(a: A) -> bool { a.field::<bool> && 0 } // recovery witness.
