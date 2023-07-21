script;

fn main() -> u64 {
    if t() {
        111
    } else {
        asm() {
            blob i262144;
        }
        222
    }
}

fn t() -> bool {
    asm() {
        one: bool
    }
}
