script;

fn main() {
    let _x = asm (r1, r2: 0, r3: 0) {
        modd r1 r2 r3;
        r1: u64
    };
}
