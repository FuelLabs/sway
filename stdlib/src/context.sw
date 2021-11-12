library context;

pub fn this_id() -> b256 {
    asm() {
            fp: b256
        }
}

pub fn msg_value() -> u64 {
    asm() {
        bal: u64
    }
}

pub fn msg_token_id() -> b256 {
    asm(token_id) {
        addi token_id fp i32;
        token_id: b256
    }
}
