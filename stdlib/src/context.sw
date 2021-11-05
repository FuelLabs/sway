library context;

struct Context {
    id: b256,
    color: b256,
}

impl Context {
    // returns the contract ID (analgous to calling `this.address` in solidity).
    fn this(self) -> b256 {
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