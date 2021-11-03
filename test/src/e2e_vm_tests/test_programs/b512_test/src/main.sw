script;
    // if test passes, return true

use std::hash::HashMethod;
use std::hash::hash_pair

/// Stores two b256s in contiguous memory. Guaranteed to be contiguous for things like
/// ECR.
struct B512 {
    hi: b256,
    lo: b256,
}

impl B512 {
    /// Initializes a blank B512
    fn new() -> B512 {
        let hi = asm(rhi) {
            mv rhi sp;
            cfei i32;
            rhi: b256
        };

        let lo = asm(rlo) {
            mv rlo sp;
            cfei i32;
            rlo: b256
        };

        B512 {
            hi: hi,
            lo: lo
        }
    }

    //

    fn from_b256(hi: b256, lo: b256) -> B512 {
        // copy the two given b256s into contiguous stack memory
        // this involves grabbing the stack pointer, extending the stack by 256 bits,
        // using MCP to copy hi into first ptr
        // repeat w/ second ptr

        let hi = asm(r1: hi, rhi) {
            mv rhi sp;
            cfei i32;
            // now $rhi is pointing to blank uninitialized, but allocated, memory
            mcpi rhi r1 i32;
            rhi: b256
        };

        let lo = asm(r1: lo, rlo) {
            mv rlo sp;
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

    fn new(pointer: b256) -> B512 {
        let lower_bits: b256 = asm(r1, r2) {
            move r2 sp;
            addi r1 r2 i32;
            r1: b256
        };

        B512 {
            hi: pointer,
            lo: lower_bits,
        }
    }
}

fn main() {

}