script;

fn main() {
    asm() {

    };

    asm(r1: 5, r2: 5) {
        add r1 r1 r2;
        add r2 r2 r2;
    };

}
