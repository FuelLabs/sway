contract;

trait MyTrait {
    fn foo();
}

abi MyAbi : MyTrait {
    fn bar();
}

// The implementation of MyAbi for Contract must also implement MyTrait
impl MyAbi for Contract {
    fn bar() { }
}