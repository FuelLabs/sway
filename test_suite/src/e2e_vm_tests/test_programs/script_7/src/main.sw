contract;
// this file tests a basic contract
struct InputStruct { field_1: bool, field_2: u64 }

abi MyContract {
  fn foo(a: u64);
  fn bar(a: InputStruct);
}


impl MyContract for Contract {
  fn foo(a: u64) {
  
  }
  fn bar(a: InputStruct){
    let status_code = if a.field_1 { "okay" } else { "fail" };
  }
}

fn calls_contract() {
  let x = abi(MyContract, 0x0000_0000_0000_0000);
}
