script;

fn main() -> u64 {
    asm(r1: 0, r2: 0, r3, r4) {
        jmp r1;
        ji i5;
        jne r1 r2 r3;
        jnei r1 r2 i5;
        jnzi r1 i5;
        ret r1;
        retd r1 r2;
        rvrt r1;
    };
    0
}
