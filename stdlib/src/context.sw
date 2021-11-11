library context;

// @todo review this. do we need to store values in struct, or just access methods?
// if we only need the methods, why not just expose functions? ie:

// fn contract_id() -> b256 {
//     asm() {
//             fp: b256
//         }
// }

struct Context {
    // id: b256
    empty: (),
}

struct Msg {
    value: u64,
    token_id: b256, // new name for `color` ?
}

impl Context {
    fn new() -> Context {
        Context {
            // id: 0x0000000000000000000000000000000000000000000000000000000000000000,
            empty: (),
        }
    }
    // returns the contract ID (analgous to calling `this.address` in solidity).
    fn id(self) -> b256 {
        asm() {
            fp: b256
        }
    }
}

impl Msg {
    fn new() -> Msg {
        Msg {
            value: 0,
            token_id: 0x0000000000000000000000000000000000000000000000000000000000000000,
        }
    }

    // returns the value of coins contained in the msg
    fn value(self) -> u64 {
        asm(value) {
            bal: u64
        }
    }

    // returns the token_id of forwarded coins.
    fn token_id(self) -> b256 {
        asm(token_id) {
            addi token_id fp i32;
            token_id: b256
        }
    }
}