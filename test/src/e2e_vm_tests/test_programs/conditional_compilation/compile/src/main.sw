script;

// These baddies should not be compiled
#[cfg(program_type = "predicate")]
const VALUE: str[3] = "bad";
#[cfg(program_type = "contract")]
const VALUE: str[3] = "bad";
#[cfg(program_type = "library")]
const VALUE: str[3] = "bad";

configurable {
  // Only compiles for FVM
  #[cfg(target = "fuel")]
  CFG_VALUE: u64 = 40,
  // Only compiles for EVM
  #[cfg(target = "evm")]
  CFG_VALUE: () = (),
  // Never compiles
  #[cfg(target = "fuel")]
  #[cfg(target = "evm")]
  CFG_VALUE: () = (),
}

#[cfg(program_type = "script")]
#[cfg(target = "fuel")]
const VALUE: u64 = 40;
#[cfg(program_type = "script")]
#[cfg(target = "evm")]
const VALUE: () = ();
#[cfg(program_type = "script")]
#[cfg(target = "fuel")]
#[cfg(target = "evm")]
const VALUE: () = ();

struct MyStruct {
  #[cfg(target = "fuel")]
  value: u64,
  #[cfg(target = "evm")]
  value: (),
  #[cfg(target = "fuel")]
  #[cfg(target = "evm")]
  value: (),
}

enum MyEnum {
  #[cfg(target = "fuel")]
  one: u64,
  #[cfg(target = "evm")]
  one: (),
  #[cfg(target = "fuel")]
  #[cfg(target = "evm")]
  one: (),
}

trait MyTrait {
  #[cfg(target = "fuel")]
  fn new(val: u64) -> Self;
  #[cfg(target = "evm")]
  fn new(val: ()) -> Self;
  #[cfg(target = "fuel")]
  #[cfg(target = "evm")]
  fn new(val: ()) -> Self;

  #[cfg(target = "fuel")]
  fn val(self) -> u64;
  #[cfg(target = "evm")]
  fn val(self) -> ();
  #[cfg(target = "fuel")]
  #[cfg(target = "evm")]
  fn val(self) -> ();
}

impl MyTrait for MyStruct {
  #[cfg(target = "fuel")]
  fn new(val: u64) -> Self {
    MyStruct { value: val }
  }
  #[cfg(target = "evm")]
  fn new(val: ()) -> Self {
    MyStruct { value: val }
  }
  #[cfg(target = "fuel")]
  #[cfg(target = "evm")]
  fn new(val: ()) -> Self {
    MyStruct { value: val }
  }

  #[cfg(target = "fuel")]
  fn val(self) -> u64 {
    self.value
  }
  #[cfg(target = "evm")]
  fn val(self) -> () {
    self.value
  }
  #[cfg(target = "fuel")]
  #[cfg(target = "evm")]
  fn val(self) -> () {
    self.value
  }
}

#[cfg(target = "fuel")]
fn main() -> u64 {
  let foo = MyStruct::new(VALUE);
  foo.val()
}
#[cfg(target = "evm")]
fn main() {
  VALUE
}
#[cfg(target = "fuel")]
#[cfg(target = "evm")]
fn main() {
  VALUE
}
