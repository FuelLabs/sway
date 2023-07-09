// This should result in an error saying that the method signature of the
// implementation does not match the declaration.

contract;

abi MyContract {
    fn foo(x: u64) -> str;
    fn bar() -> u32;
    fn baz() -> u64;
}

impl MyContract for Contract {
    fn foo(s: str) -> str {
        s
    }

    fn bar() -> u64 {
        0
    }

    fn baz() { // No return type here
    }
}
