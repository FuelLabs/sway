contract;

abi MyAbi { }
{
    fn impl_method() -> u64 { 42 }
}

impl MyAbi for Contract { }

#[test]
fn tests() {
    let caller = abi(MyAbi, CONTRACT_ID);
    assert(caller.impl_method() == 42)
}
