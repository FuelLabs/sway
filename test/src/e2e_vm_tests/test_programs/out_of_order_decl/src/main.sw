script;
// This test tests two-pass compilation and allowing usages before declarations.

fn main() -> bool {
  let a = 42;
  // fn before decl
  let x = the_number_five();
  // enum before decl
  let z = AnEnum::Variant;
  // struct before decl
  let y = FuelStruct {
    a: true,
    b: false
  };
  return true;
}

struct FuelStruct {
  a: bool,
  b: bool
}

fn the_number_five() -> u64 {
  5
}

enum AnEnum {
  Variant: (),
}

// trait before decl 
impl FuelTrait for u64 {
  fn foo() -> bool {
    true
  }
}

trait FuelTrait {
  fn foo() -> bool;
}
