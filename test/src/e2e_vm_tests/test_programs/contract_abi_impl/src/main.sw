contract;
// this file tests a basic contract and contract call
struct InputStruct { field_1: bool, field_2: u64 }

abi MyContract {
  fn foo(gas: u64, coin: u64, color: b256, input: InputStruct);
} {
  fn baz(gas: u64, coin: u64, color: b256, input: bool) { } 
}


impl MyContract for Contract {
  fn foo(gas: u64, coin: u64, color: b256, input: InputStruct) {
    let status_code = if input.field_1 { "okay" } else { "fail" };
    calls_other_contract();
  }
}

fn calls_other_contract() {
  let x = abi(MyContract, 0x0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
  // commenting this out for now since contract call asm generation is not yet implemented
  let color = 0x0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
  let input = InputStruct {
    field_1: true,
    field_2: 3,
  };
  x.foo(5, 5, color, input);
}
