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
    let x = abi(MyContract, 0x79fa8779bed2f36c3581d01c79df8da45eee09fac1fd76a5a656e16326317ef0);
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    x.foo {
        gas: 5000, coins: 0, asset_id: asset_id
    }
    (true, 3);
    0
}
