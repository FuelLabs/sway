library;

trait MyTrait {
    fn foo();
}

abi MyAbi : MyTrait {
    fn bar();
} {
    fn baz() { Self::foo() }
}

pub fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(MyAbi, contract_address);
    // this is ok, bar is part of public interface
    caller.bar {
        gas: 10000,
        coins: 0,
        asset_id: 0x0000000000000000000000000000000000000000000000000000000000000000,
    }();
    // this is an error, foo is NOT part of public interface
    // TODO: We need to improve the error message here.
    //       Currently it says that a method is not found for type,
    //       and at the same time it lists the method as implemented for the type,
    //       which is confusing.
    //       We should instead say that the method is not part of the ABI's public interface.
    caller.foo {
        gas: 10000,
        coins: 0,
        asset_id: 0x0000000000000000000000000000000000000000000000000000000000000000,
    }();
}
