contract;

trait MySuperTrait {
    fn method() -> u64;
}

abi MyAbi : MySuperTrait {
    //fn method() -> u64;
}

impl MySuperTrait for Contract {
    fn method() -> u64 { 42 }
}

impl MyAbi for Contract {}

#[test]
fn tests() {
    let caller = abi(MyAbi, CONTRACT_ID);
    assert(caller.method() == 0xBAD)
}
