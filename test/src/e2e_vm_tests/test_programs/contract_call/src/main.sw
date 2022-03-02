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
    let x = abi(MyContract, 0x4486a2fec1fd4c76c7bab957e45d8b89d0b082c4ecca62083a5e56e9f1234a61);
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    x.foo {
        gas: 5000, coins: 0, asset_id: asset_id
    }
    (true, 3);
    0
}
