contract;
// this file tests a basic contract and contract call
struct InputStruct {
    field_1: bool,
    field_2: u64,
}

abi MyContract {
    fn foo(field_1: bool, field_2: u64) -> InputStruct;
} {
    fn baz(input: bool) {
    }
}

impl MyContract for Contract {
    fn foo(field_1: bool, field_2: u64) -> InputStruct {
        let status_code = if field_1 {
            "okay"
        } else {
            "fail"
        };
        calls_other_contract()
    }
}

fn calls_other_contract() -> InputStruct {
    let x = abi(MyContract, 0x0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    // commenting this out for now since contract call asm generation is not yet implemented
    let asset_id = 0x0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
    x.foo {
        gas: 5, coins: 5, asset_id: asset_id
    }
    (true, 3)
}
