contract;

struct S {
    x: u64,
}

abi MyContract {
    fn foo(s: S) -> (S);
}

impl MyContract for Contract {
    fn foo(s: S) -> (S) {
        (s)
    }
}
