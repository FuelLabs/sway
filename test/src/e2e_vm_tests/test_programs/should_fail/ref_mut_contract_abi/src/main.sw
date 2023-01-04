contract;

abi MyAbi {
    fn foo(ref mut x: u64);
}

impl MyAbi for Contract {
    fn foo(ref mut x: u64) {

    }
}
