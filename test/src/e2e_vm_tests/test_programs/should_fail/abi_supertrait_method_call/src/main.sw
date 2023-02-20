script;

use std::constants::ZERO_B256;

trait MyTrait {
    fn foo();
}

abi MyAbi : MyTrait {
    fn bar();
} {
    fn baz() { Self::foo() }
}

fn main() {
    let contract_address = 0x9299da6c73e6dc03eeabcce242bb347de3f5f56cd1c70926d76526d7ed199b8b;
    let caller = abi(MyAbi, contract_address);
    // this is ok, bar is part of public interface
    caller.bar {
        gas: 10000,
        coins: 0,
        asset_id: ZERO_B256,
    }();
    // this is an error, foo is NOT part of public interface
    caller.foo {
        gas: 10000,
        coins: 0,
        asset_id: ZERO_B256,
    }();
}
