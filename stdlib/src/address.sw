library address;

// @todo consider using tuple structs if they land.
// ie: pub struct Address(b256);
// let addr = Address(0x8900c5bec4ca97d4febf9ceb4754a60d782abbf3cd815836c1872116f203f861);
// usage:
pub struct Address {
   inner: b256
}

pub trait From {
    fn from(self) -> Self;
}
// {
//     fn into(Self) -> self {
//         self
//     }
// }

impl From for Address {
    fn from(self) -> Address {
        let addr = asm(r1: self, inner) {
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