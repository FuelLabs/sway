library types;

// Stores two b256s in contiguous memory.
// Guaranteed to be contiguous for things like ECR.
struct B512 {
    hi: b256,
    lo: b256,
}

// temp
pub fn build_from_b256s(hi: b256, lo: b256) -> B512 {
    let hi = asm(r1: hi, rhi, r2: 32) {
            move rhi sp; // move stack pointer to rhi
            cfei i32;  // extend call frame by 32 bytes to allocate more memory. now $rhi is pointing to blank, uninitialized (but allocated) memory
            // addi r5 zero i32;
            mcp rhi r1 r2;
            rhi: b256
        };

        let lo = asm(r1: lo, rlo, r2: 32) {
            move rlo sp;
            cfei i32;
            // now $rlo is pointing to blank memory that we can use
            mcp rlo r1 r2;
            rlo: b256
        };

        B512 {
            hi: hi,
            lo: lo
        }
}

impl B512 {
    // Initializes a blank B512
    fn new() -> B512 {
        let hi = asm(rhi) {
            move rhi sp;
            cfei i32;
            rhi: b256
        };

        let lo = asm(rlo) {
            move rlo sp;
            cfei i32;
            rlo: b256
        };

        B512 {
            hi: hi,
            lo: lo
        }
    }

    fn from_b256(hi: b256, lo: b256) -> B512 {
        // copy the two given b256s into contiguous stack memory
        // this involves grabbing the stack pointer, extending the stack by 256 bits,
        // using MCP to copy hi into first ptr
        // repeat w/ second ptr

        let hi = asm(r1: hi, rhi, r2: 32) {
            move rhi sp; // move stack pointer to rhi
            cfei i32;  // extend call frame by 32 bytes to allocate more memory. now $rhi is pointing to blank, uninitialized (but allocated) memory
            mcp rhi r1 r2; // refactor to use mcpi when implemented!
            rhi: b256
        };

        let lo = asm(r1: lo, rlo, r2) {
            move rlo sp;
            cfei i32;
            // now $rlo is pointing to blank memory that we can use
            mcp rlo r1 r2; // refactor to use mcpi when implemented!
            rlo: b256
        };

        B512 {
            hi: hi,
            lo: lo
        }
    }
}