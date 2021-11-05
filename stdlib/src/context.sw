library context;

struct Context {
    id: b256,
    color: b256,
}

impl Context {
    fn new() -> Context {
        Context {
            id: 0x0000000000000000000000000000000000000000000000000000000000000000,
            color: 0x0000000000000000000000000000000000000000000000000000000000000000,
        }
    }
    // returns the contract ID (analgous to calling `this.address` in solidity).
    fn id(self) -> b256 {
        asm() {
            fp: b256
        }
    }

    // returns the color of forwarded coins.
    fn color(self) -> b256 {
        asm(r1) {
            addi r1 fp i32;
            r1: b256
        }
    }
}