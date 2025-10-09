contract;

impl Contract {
    fn impl_method() -> u64 { 42 }
}

#[test]
fn tests() {
    let caller = abi(ContractAbiAutoImplAbi, CONTRACT_ID);
    assert(caller.impl_method() == 42)
}
