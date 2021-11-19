library chain;
dep chain/auth;
use ::ops::*;

// When generics land, these will be generic.
pub fn log_u64(val: u64) {
  asm(r1: val) {
    log r1 zero zero zero;
  }
}

pub fn log_u32(val: u32) {
  asm(r1: val) {
    log r1 zero zero zero;
  }
}

pub fn log_u16(val: u16) {
  asm(r1: val) {
    log r1 zero zero zero;
  }
}

pub fn log_u8(val: u8) {
  asm(r1: val) {
    log r1 zero zero zero;
  }
}


/// Context-dependent:
/// will panic if used in a predicate
/// will revert if used in a contract
pub fn panic(code: u64) {
  asm(r1: code) {
    rvrt r1;
  }
}

/// Assert that a value is true
pub fn assert(a: bool) {
    if not(a) {
        panic(0);
    } else {
        ()
    }
}
