script;
// this file tests a contract call from a script
struct InputStruct {
    field_1: bool,
    field_2: u64,
}

abi MyContract {
    fn foo(field_1: bool, field_2: u64);
} {
    fn baz(field_1: bool) {
    }
}

fn main() -> u64 {
    let x = abi(MyContract, 0x3dba0a4455b598b7655a7fb430883d96c9527ef275b49739e7b0ad12f8280eae);
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    x.foo {
        gas: 5000, coins: 0, asset_id: asset_id
    }
    (true, 3);
    0
}
