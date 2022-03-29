library chain;
dep chain/auth;

use ::panic::panic;

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
