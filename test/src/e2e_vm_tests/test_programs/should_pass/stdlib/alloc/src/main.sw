script;

use core::num::*;
use std::alloc::*;
use std::intrinsics::*;
use std::assert::assert;

fn lw(ptr: raw_ptr) -> u64 {
    asm(r1: ptr, res) {
        lw res r1 i0;
        res: u64
    }
}

fn sw(ptr: raw_ptr, val: u64) {
    asm(r1: ptr, val: val) {
        sw r1 val i0;
    };
}

fn heap_ptr() -> raw_ptr {
    asm(ptr) {
        addi ptr hp i1;
        ptr: raw_ptr
    }
}

fn main() -> bool {
    // Allocate zero
    let hp = heap_ptr();
    let buf = alloc::<u64>(0);
    assert(buf.ptr() == hp);
    assert(heap_ptr() == hp);

    // Allocate some memory
    let hp = heap_ptr();
    let buf = alloc::<u64>(1);
    assert(buf.ptr() == hp.sub::<u64>(1));
    assert(heap_ptr() == hp.sub::<u64>(1));

    // Read from it
    let val = lw(buf.ptr());
    assert(val == 0);

    // Write to it
    let val = u64::max();
    sw(buf.ptr(), val);
    assert(lw(buf.ptr()) == val);

    // Grow it
    let hp = heap_ptr();
    let buf = realloc::<u64>(buf, 2);
    assert(buf.ptr() == hp.sub::<u64>(2));
    assert(heap_ptr() == hp.sub::<u64>(2));

    // Make sure that reallocating an old allocation of size 0 does not cause a
    // panic.
    let hp = heap_ptr();
    let buf = alloc::<u64>(0);
    let buf = realloc::<u64>(buf, 2);
    assert(buf.ptr() == hp.sub::<u64>(2));
    assert(heap_ptr() == hp.sub::<u64>(2));

    true
}
