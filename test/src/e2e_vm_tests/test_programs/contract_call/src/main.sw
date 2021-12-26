script;
// this file tests a contract call from a script
struct InputStruct {
    field_1: bool,
    field_2: u64,
}

abi MyContract {
    fn foo(gas: u64, coin: u64, asset_id: b256, input: InputStruct);
} {
    fn baz(gas: u64, coin: u64, asset_id: b256, input: bool) {
    }
}

fn main() -> u64 {
    let x = abi(MyContract, 0x6c626fddd128e24e6805fe1779779f14097d34086c571dd8df1c78ac4bb9a78b);
    let asset_id = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    let input = InputStruct {
        field_1: true,
        field_2: 3,
    };
    x.foo(5000, 0, asset_id, input);
    0
}
