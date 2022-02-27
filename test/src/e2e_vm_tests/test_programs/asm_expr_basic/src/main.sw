script;

use std::chain::*;

// This file tests different kinds of ASM generation and parsing.

fn blockheight() -> u64 {
    asm(r1) {
        bhei r1;
        r1: u64
    }
}

struct GasCounts {
    global_gas: u64,
    context_gas: u64,
}

fn get_gas() -> GasCounts {
    GasCounts {
        global_gas: asm() {
            ggas
        },
        context_gas: asm() {
            cgas
        }
    }
}

fn main() -> u32 {
    let block_height = blockheight();
    let remaining_gas = get_gas();

    // Test the spelling of all special registers
    let zero = asm() { zero };
    assert(zero == 0);

    let one = asm() { one };
    assert(one == 1);

    let of = asm() { of };
    assert(of == 0);

    let pc = asm() { pc };

    let ssp = asm() { ssp };

    let sp = asm() { sp };

    let fp = asm() { fp };

    let hp = asm() { hp };

    let err = asm() { err };
    assert(err == 0);

    let ggas = asm() { ggas };

    let cgas = asm() { cgas };

    let bal = asm() { bal };

    let is = asm() { is };
    
    let ret = asm() { ret };

    let retl = asm() { retl };

    let flag = asm() { flag };

    return 6u32;
}
