script;

fn main() -> u64 {
    asm() {
        blob i262144;
    }
    if t() {
        111
    } else {
        222
    }
}

fn t() -> bool {
    asm() {
        one: bool
    }
}
