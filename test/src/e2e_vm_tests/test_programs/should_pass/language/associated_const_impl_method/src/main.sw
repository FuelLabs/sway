script;

struct Struct { }

impl Struct {
    const ID: u64 = 1;

    fn foo(self) -> u64 {
        Self::ID
    }
}

fn main() -> u64 {
  let s = Struct {};
  s.foo()
}
