library address;

struct Address {
   inner: b256
}

impl Address {
  fn from_b256(a: b256) -> Address {

      let addr = asm(r1: a, inner) {
            move inner sp; // move stack pointer to inner
            cfei i32;  // extend call frame by 32 bytes to allocate more memory. now $inner is pointing to blank, uninitialized (but allocated) memory
            mcpi inner r1 i32; // refactor to use mcpi when implemented!
            inner: b256
        };
      Address {
          inner: addr,
      }
  }
}