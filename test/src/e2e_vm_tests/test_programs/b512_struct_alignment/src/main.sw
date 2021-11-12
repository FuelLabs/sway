script;

// Stores two b256s in contiguous memory.
// Guaranteed to be contiguous for things like ECR.
struct B512 {
    hi: b256,
    lo: b256,
}

// temp
pub fn build_from_b256s(hi: b256, lo: b256) -> B512 {
    let hi = asm(r1: hi, rhi) {
            move rhi sp; // move stack pointer to rhi
            cfei i32;  // extend call frame by 32 bytes to allocate more memory. now $rhi is pointing to blank, uninitialized (but allocated) memory
            // addi r5 zero i32;
            mcpi rhi r1 i32;
            rhi: b256
        };

        let lo = asm(r1: lo, rlo) {
            move rlo sp;
            cfei i32;
            // now $rlo is pointing to blank memory that we can use
            mcpi rlo r1 i32;
            rlo: b256
        };

        B512 {
            hi: hi,
            lo: lo
        }
}

fn main() -> bool {
    let hi_bits: b256 = 0x7777777777777777777777777777777777777777777777777777777777777777;
    let lo_bits: b256 = 0x5555555555555555555555555555555555555555555555555555555555555555;

    let b: B512 = build_from_b256s(hi_bits, lo_bits);

    b.lo == lo_bits && b.hi == hi_bits
}
