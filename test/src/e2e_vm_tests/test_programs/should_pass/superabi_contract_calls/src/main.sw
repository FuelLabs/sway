// Inheritance graph
//          MySuperAbi
//              |
//            MyAbi

contract;

abi MySuperAbi {
    fn superabi_method() -> u64;
}

abi MyAbi : MySuperAbi {
    fn abi_method() -> u64;
}

impl MySuperAbi for Contract {
    fn superabi_method() -> u64 { 42 }
}

impl MyAbi for Contract {
    fn abi_method() -> u64 { 43 }
}

#[test]
fn tests() {
    let caller = abi(MyAbi, CONTRACT_ID);
    assert(caller.abi_method() == 43);
    // superABI methods become ABI methods too
    assert(caller.superabi_method() == 42)
}