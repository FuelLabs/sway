contract;

abi Abi {
    fn return_via_contract_ret(x: u8) -> u64;
}

impl Abi for Contract {
    fn return_via_contract_ret(x: u8) -> u64 {
        match x {
            1 => {
                let ret: raw_slice = encode::<u64>(100);
                __contract_ret(ret.ptr(), ret.len::<u8>());
                __revert(100);
            },
            2 => {
                let ret: raw_slice = encode::<u64>(200);
                __contract_ret(ret.ptr(), ret.len::<u8>());
                __revert(200);
            },
            _ => __revert(0xaaa),
        }
    }
}

#[test]
fn test() {
    let caller = abi(Abi, CONTRACT_ID);

    let res = caller.return_via_contract_ret(1);
    assert_eq(100, res);

    let res = caller.return_via_contract_ret(2);
    assert_eq(200, res);
}
