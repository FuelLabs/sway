script;

struct Struct { }

impl Struct {
    const ID: u32 = 1;

    fn foo(self) -> u32 {
      Self::ID
    }
}

fn main() -> u32 {
  let s = Struct{};
  s.foo()
}
