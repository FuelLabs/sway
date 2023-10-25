script;

struct Struct { }

impl Struct {
    fn foo(self) -> u32 {
        Self::ID
    }

    const ID: u32 = 1;
}

fn main() -> u32 {
  let s = Struct {};
  s.foo()
}
