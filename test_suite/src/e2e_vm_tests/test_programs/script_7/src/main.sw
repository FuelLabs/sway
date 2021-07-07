contract;
// this file tests a basic contract
struct InputStruct { field_1: bool, field_2: u64 }
struct OutputStruct { status_code: str[4] }

trait MyContract {
  fn foo(a: u64) -> bool;
  fn bar(a: InputStruct) -> OutputStruct;
}


impl MyContract for Self {
  fn foo(a: u64) -> bool {
    a > 5
  }
  fn bar(a: InputStruct) -> OutputStruct {
    let status_code = if a.field_1 { "okay" } else { "fail" };
    OutputStruct { status_code: status_code }
  }
}
