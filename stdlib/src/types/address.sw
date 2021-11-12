library address;

struct Address {
   inner: b256
}

impl Address {
  fn new(a: b256) -> Self {
      Address {
          inner: a,
      }
  }
}