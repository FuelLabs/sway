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
    return 6u32;
}
