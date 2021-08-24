script;
// this file tests a contract call from a script
struct InputStruct { field_1: bool, field_2: u64 }

abi MyContract {
  fn foo(gas: u64, coin: u64, color: byte32, input: InputStruct);
} {
  fn baz(gas: u64, coin: u64, color: byte32, input: bool) { } 
}

fn main () {
  let x = abi(MyContract, 0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);
  // commenting this out for now since contract call asm generation is not yet implemented
  let color = 0x7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777;
  let input = InputStruct {
    field_1: true,
    field_2: 3,
  };
  x.foo(5000, 0, color, input);
}
