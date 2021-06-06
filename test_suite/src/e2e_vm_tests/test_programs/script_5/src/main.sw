script;
// This tests importing other files.

dep a_dependency;
// TODO:
// find missing imports in other modules
// figure out why it can't find module `foo`
// don't err "missing main func" if there was an error in the main func
// figure out str type not working

use foo::Foo;

fn main() -> bool {
  let foo = Foo {
    foo: "foo",
  };
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
