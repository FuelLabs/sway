script;

fn main() -> raw_ptr {
    let ptr = asm(r1) { r1: raw_ptr };
    ptr
}
