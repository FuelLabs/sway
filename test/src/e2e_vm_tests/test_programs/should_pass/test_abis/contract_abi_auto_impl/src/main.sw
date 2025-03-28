contract;

impl Contract {
    fn impl_method() -> u64 { 42 }
}

#[test]
fn tests() {
    let caller = abi(_AnonymousAbi_1, CONTRACT_ID);
    assert(caller.impl_method() == 42)
}
