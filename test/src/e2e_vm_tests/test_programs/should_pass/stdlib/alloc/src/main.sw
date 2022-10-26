script;

use core::num::*;
use std::alloc::*;
use std::intrinsics::*;
use std::context::registers::*;
use std::assert::assert;

fn lw(ptr: raw_ptr) -> u64 {
    asm(r1: ptr) {
        lw r1 r1 i0;
        r1: u64
    }
}

fn sw(ptr: raw_ptr, val: u64) {
    asm(r1: ptr, val: val) {
        sw r1 val i0;
    };
}

fn main() -> bool {
    let hp_start = heap_ptr();

    // Allocate zero
    let hp = heap_ptr();
    let ptr = alloc(0);
    assert(ptr == hp.add(1));
    assert(heap_ptr() == hp);

    // Allocate some memory
    let hp = heap_ptr();
    let ptr = alloc(8);
    assert(ptr == hp.sub(8).add(1));
    assert(heap_ptr() == hp.sub(8));

    // Read from it
    let val = lw(ptr);
    assert(val == 0);

    // Write to it
    let val = ~u64::max();
    sw(ptr, val);
    assert(lw(ptr) == val);

    // Grow it
    let hp = heap_ptr();
    let ptr = realloc(ptr, 8, 16);
    assert(ptr == hp.sub(16).add(1));
    assert(heap_ptr() == hp.sub(16));

    // Make sure that reallocating an old allocation of size 0 does not cause a
    // panic. 
    let hp = heap_ptr();
    let ptr = alloc(0);
    let ptr = realloc(ptr, 0, 16);
    assert(ptr == hp.sub(16).add(1));
    assert(heap_ptr() == hp.sub(16));

    true
}
