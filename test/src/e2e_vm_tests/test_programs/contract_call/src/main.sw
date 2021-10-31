script;
// this file tests a contract call from a script
struct InputStruct {
    field_1: bool,
    field_2: u64,
}

abi MyContract {
    fn foo(gas: u64, coin: u64, color: b256, input: InputStruct);
} {
    fn baz(gas: u64, coin: u64, color: b256, input: bool) {
    }
}

fn main() -> u64 {
    let x = abi(MyContract, 0x8f0f0806a879ec62f5afd12a76e5f3afbb1e5b27651ca8a633963ea981a08219);
    // commenting this out for now since contract call asm generation is not yet implemented
    let color = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
    let input = InputStruct {
        field_1: true,
        field_2: 3,
    };
    x.foo(5000, 0, color, input);
    0
}
