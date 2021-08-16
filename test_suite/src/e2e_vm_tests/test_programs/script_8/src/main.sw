script;
// this file tests a contract call from a script
struct InputStruct { field_1: bool, field_2: u64 }

abi MyContract {
  fn foo(gas: u64, coin: u64, color: byte32, input: InputStruct);
} {
  fn baz(gas: u64, coin: u64, color: byte32, input: bool) { } 
}

fn main () {
  let x = abi(MyContract, 0x0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
  // commenting this out for now since contract call asm generation is not yet implemented
  let color = 0x0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
  x.foo(5, 0, color, 7);
}
