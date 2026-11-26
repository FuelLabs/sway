contract;

use std::{
    flags::{
        disable_panic_on_overflow, disable_panic_on_unsafe_math, set_flags,
    },
    registers::{error, overflow},
};

abi Probe {
    fn noop_clears_overflow(x: u64, first: bool) -> u64;
    fn move_clears_overflow(x: u64, first: bool) -> u64;
    fn noop_clears_error(x: u64, first: bool) -> u64;
    fn move_clears_error(x: u64, first: bool) -> u64;
}

impl Probe for Contract {
    fn noop_clears_overflow(x: u64, first: bool) -> u64 {
        let old = disable_panic_on_overflow();
        if first {
            let _unused = asm(a: x, b: u64::max(), r) {
                add r a b;
                noop;
                r: u64
            };
        } else {
            let _unused = asm(a: x, b: u64::max() - 1, r) {
                add r a b;
                noop;
                r: u64
            };
        }
        let status = overflow();
        set_flags(old);
        status
    }

    fn move_clears_overflow(x: u64, first: bool) -> u64 {
        let old = disable_panic_on_overflow();
        if first {
            let _unused = asm(a: x, b: u64::max(), r) {
                add r a b;
                move r r;
                r: u64
            };
        } else {
            let _unused = asm(a: x, b: u64::max() - 1, r) {
                add r a b;
                move r r;
                r: u64
            };
        }
        let status = overflow();
        set_flags(old);
        status
    }

    fn noop_clears_error(x: u64, first: bool) -> u64 {
        let old = disable_panic_on_unsafe_math();
        if first {
            let _unused = asm(a: x, b: 0, r) {
                div r a b;
                noop;
                r: u64
            };
        } else {
            let _unused = asm(a: x, b: 0, r) {
                div r a b;
                noop;
                r: u64
            };
        }
        let status = error();
        set_flags(old);
        status
    }

    fn move_clears_error(x: u64, first: bool) -> u64 {
        let old = disable_panic_on_unsafe_math();
        if first {
            let _unused = asm(a: x, b: 0, r) {
                div r a b;
                move r r;
                r: u64
            };
        } else {
            let _unused = asm(a: x, b: 0, r) {
                div r a b;
                move r r;
                r: u64
            };
        }
        let status = error();
        set_flags(old);
        status
    }
}

#[test]
fn preserves_status_clears_across_joins() {
    let p = abi(Probe, CONTRACT_ID);

    assert(p.noop_clears_overflow(2, true) == 0);
    assert(p.noop_clears_overflow(2, false) == 0);
    assert(p.move_clears_overflow(2, true) == 0);
    assert(p.move_clears_overflow(2, false) == 0);
    assert(p.noop_clears_error(2, true) == 0);
    assert(p.noop_clears_error(2, false) == 0);
    assert(p.move_clears_error(2, true) == 0);
    assert(p.move_clears_error(2, false) == 0);
}

