contract;
// this file tests a basic contract
struct InputStruct { field_1: bool, field_2: u64 }

trait MyContract {
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
