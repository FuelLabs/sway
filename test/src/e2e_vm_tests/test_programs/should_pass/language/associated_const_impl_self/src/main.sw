script;

struct Struct { }

impl Struct {
    const ID: u32 = 3;

    fn foo(self) -> u32 {
        Self::ID
    }
}

struct StructOrder { }

impl StructOrder {
    fn foo(self) -> u32 {
        Self::ID
    }

    // The const declaration comes after the const usage.
    const ID: u32 = 5;
}

fn main() -> u32 {
  let s = Struct {};
  let so = StructOrder {};

  s.foo() + so.foo()
}
