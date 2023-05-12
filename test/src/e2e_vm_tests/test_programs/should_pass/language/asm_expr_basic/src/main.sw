script;

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
        global_gas: asm() { ggas },
        context_gas: asm() { cgas },
    }
}

fn main() -> u32 {
    let _block_height = blockheight();
    let _remaining_gas = get_gas();

    // Test the spelling of all special registers
    let zero = asm() { zero };
    assert(zero == 0);

    let one = asm() { one };
    assert(one == 1);

    let of = asm() { of };
    assert(of == 0);

    let _pc = asm() { pc };

    let _ssp = asm() { ssp };

    let _sp = asm() { sp };

    let _fp = asm() { fp };

    let _hp = asm() { hp };

    let err = asm() { err };
    assert(err == 0);

    let _ggas = asm() { ggas };

    let _cgas = asm() { cgas };

    let _bal = asm() { bal };

    let _is = asm() { is };

    let _ret = asm() { ret };

    let _retl = asm() { retl };

    let _flag = asm() { flag };

    let _x = asm(r1, r2: 2, r3: 1) {
        mod  r1 r2 r3;
        r1: u64
    };

    return 6u32;
}
