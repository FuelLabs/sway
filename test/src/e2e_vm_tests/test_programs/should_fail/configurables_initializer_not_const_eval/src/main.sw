script;

fn not_const_eval() -> u64 {
    asm() { fp: u64 }
}

configurable {
    CONFIG: u64 = not_const_eval(),
}

fn main() {}