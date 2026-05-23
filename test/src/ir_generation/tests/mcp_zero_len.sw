// target-fuelvm
script;

fn main() -> u64 {
    let src: u64 = 42;
    let mut dst: u64 = 0;
    asm(dst_ptr: dst, src_ptr: src, zero: 0u64) {
        mcp dst_ptr src_ptr zero;
    }
    dst
}

// ::check-ir::

// check: asm(dst_ptr: $VAL, src_ptr: $VAL, zero: $VAL) -> ()
// check: mcp

// ::check-asm::
//
// not: mcp
