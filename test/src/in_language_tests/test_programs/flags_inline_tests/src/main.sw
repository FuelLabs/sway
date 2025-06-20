library;

use std::{
    flags::{
        disable_panic_on_overflow,
        disable_panic_on_unsafe_math,
        enable_panic_on_overflow,
        enable_panic_on_unsafe_math,
        panic_on_overflow_enabled,
        panic_on_unsafe_math_enabled,
        set_flags,
    },
    registers::error,
};

#[test]
fn flags_disable_panic_on_overflow() {
    let _ = disable_panic_on_overflow();
    let _bar = u64::max() + 1;
    enable_panic_on_overflow();
}

#[test]
fn flags_disable_panic_on_overflow_preserving() {
    let _ = disable_panic_on_overflow();

    let prior_flags = disable_panic_on_overflow();
    let _bar = u64::max() + 1;
    set_flags(prior_flags);

    let _bar = u64::max() + 1;

    enable_panic_on_overflow();
}

#[test]
fn flags_disable_panic_on_unsafe_math() {
    let _ = disable_panic_on_unsafe_math();

    let err = asm(r2: 1, r3: 0, r1) {
        div r1 r2 r3;
        err: u64
    };

    assert(err == 1);

    enable_panic_on_unsafe_math();
}

#[test]
fn flags_disable_panic_on_unsafe_math_preserving() {
    let _ = disable_panic_on_unsafe_math();

    let prior_flags = disable_panic_on_unsafe_math();
    let err = asm(r2: 1, r3: 0, r1) {
        div r1 r2 r3;
        err: u64
    };
    assert(err == 1);
    set_flags(prior_flags);

    let err = asm(r2: 1, r3: 0, r1) {
        div r1 r2 r3;
        err: u64
    };
    assert(err == 1);

    enable_panic_on_unsafe_math();
}

#[test]
fn test_panic_on_overflow_enabled() {
    // Enabled by default
    assert(panic_on_overflow_enabled());

    disable_panic_on_overflow();
    assert(!panic_on_overflow_enabled());

    enable_panic_on_overflow();
    assert(panic_on_overflow_enabled());
}

#[test]
fn test_panic_on_unsafe_math_enabled() {
    // Enabled by default
    assert(panic_on_unsafe_math_enabled());

    disable_panic_on_unsafe_math();
    assert(!panic_on_unsafe_math_enabled());

    enable_panic_on_unsafe_math();
    assert(panic_on_unsafe_math_enabled());
}
