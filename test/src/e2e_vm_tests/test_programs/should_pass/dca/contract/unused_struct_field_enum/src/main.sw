contract;

struct S {
    x: u64,
}

enum E {
    A: S,
}

abi MyContract {
    fn foo(s: S) -> E;
}

impl MyContract for Contract {
    fn foo(s: S) -> E {
        E::A(s)
    }
}
