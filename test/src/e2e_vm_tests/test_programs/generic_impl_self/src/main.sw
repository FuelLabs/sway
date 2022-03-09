script;

struct DoubleIdentity<T, F> {
  first: T,
  second: F
}

impl<T, F> DoubleIdentity<T, F> {
  fn new(x: T, y: F) -> DoubleIdentity<T, F> {
    DoubleIdentity {
      first: x,
      second: y
    }
  }

  fn get_first(self) -> T {
    let x: T = self.first;
    x
  }

  fn get_second(self) -> F {
    let y: F = self.second;
    y
  }
}

/*
impl DoubleIdentity<u8, u8> {
  fn add(self) -> u8 {
    self.first + self.second
  }
}
*/

fn double_identity2<T, F>(x: T, y: F) -> DoubleIdentity<T, F> {
  ~DoubleIdentity<T, F>::new(x, y)
}

fn double_identity<T, F>(x: T, y: F) -> DoubleIdentity<T, F> {
  let inner: T = x;
  DoubleIdentity {
    first: inner,
    second: y
  }
}

fn crazy<T, F>(x: T, y: F) -> F {
  let foo = DoubleIdentity {
    first: x,
    second: y,
  };
  foo.get_second()
}

fn main() -> u32 {
  let a = double_identity(true, true);
  let b = double_identity(10u32, 43u64);
  let c = double_identity2(10u8, 1u8);
  let d = DoubleIdentity {
    first: 1u8,
    second: 2u8
  };
  let e = d.get_second();
  let f: DoubleIdentity<bool, bool> = double_identity(true, true);
  let g: DoubleIdentity<u32, u64> = double_identity(10u32, 43u64);
  let h = ~DoubleIdentity<u64, bool>::new(3u64, false);
  let i = crazy(7u8, 10u8);

  b.get_first()
}
