// optimisation-inline

script;

#[inline(always)]
fn always_inline() -> u64 {
    3
}

#[inline(never)]
fn never_inline() -> u64 {
    3
}

fn main() -> u64{
    always_inline();
    never_inline()
}

// ::check-ir::

// check: fn always_inline_0() -> u64
// check: fn never_inline_1() -> u64

// ::check-asm::
// not: [call: always_inline_0]: call function
// check: [call: never_inline_1]: call function
