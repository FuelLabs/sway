contract;

// The method names "way" and "fpeu" end up with the same 4 bit function selector. This means that
// calling one of the methods on a contract will result in either of the methods being invoked
// arbitrarily.
// This is only relevant when using v0 encoding, since v1 encoding does not use
// function selectors.
abi MyContract {
    fn way() -> (bool);
    fn fpeu() -> (bool);
}

impl MyContract for Contract {
    fn way() -> bool {
        true
    }

    fn fpeu() -> bool {
        false
    }
}

#[test]
fn test() {
    let c = abi(MyContract, CONTRACT_ID);
    assert(c.way());
}
