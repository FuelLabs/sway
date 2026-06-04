// target-fuelvm
script;

fn main() -> u64 {
    let src: u64 = 42;
    let mut dst: u64 = 0;
    asm(dst_ptr: __addr_of(dst), src_ptr: __addr_of(src), zero: 0u64) {
        mcp dst_ptr src_ptr zero;
    }
    asm(dst_ptr: __addr_of(dst), src_ptr: __addr_of(src)) {
        mcpi dst_ptr src_ptr i0;
    }
    dst
}

// ::check-ir::

// check: asm(dst_ptr: $VAL, src_ptr: $VAL, zero: $VAL) -> ()
// check: mcp
// check: asm(dst_ptr: $VAL, src_ptr: $VAL) -> ()
// check: mcpi

// ::check-asm::
//
// not: mcp
// not: mcpi
