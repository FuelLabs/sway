// This should result in an error saying that the method signature of the
// implementation does not match the declaration.

contract;

abi MyContract {
    fn foo(x: u64) -> str[7];
}

impl MyContract for Contract {
    fn foo(s: str[7]) -> str[7] {
        s
    }
}
