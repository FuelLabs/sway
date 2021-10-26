library auth;
use ::ops::*;

// this can be a generic option when options land
enum Caller {
  Some: b256,
  None: (),
}

pub fn caller() -> Caller {
  // if parent is external
  if not(asm(r1) {
    gm r1 i1;
    r1: bool
  }) {
    // get the caller
    Caller::Some(asm(r1) {
      gm r1 i2;
      r1: b256
    })
  } else {
    Caller::None
  }
}
