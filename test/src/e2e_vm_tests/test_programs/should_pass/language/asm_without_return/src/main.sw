script;

fn main() {
    asm() {
    };

    asm(r1: 5, r2: 5, r3, r4) {
        add r3 r1 r2;
        add r4 r2 r2;
    };
}
