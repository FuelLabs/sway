contract;

struct S {
    x: u64,
}

abi MyContract {
    fn foo(s: S) -> [S; 1];
}

impl MyContract for Contract {
    fn foo(s: S) -> [S; 1] {
        [s]
    }
}
