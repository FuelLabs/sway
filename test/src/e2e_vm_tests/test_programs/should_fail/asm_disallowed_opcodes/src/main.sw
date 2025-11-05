library;

pub fn main() {
    asm(r1: 0, r2: 0, r3: 0, r4) {
        jmp r1;
        ji i5;
        jne r1 r2 r3;
        jnei r1 r2 i5;
        jnzi r1 i5;
        jmpb r1 i5;
        jmpf r1 i5;
        jnzb r1 r2 i5;
        jnzf r1 r2 i5;
        jneb r1 r2 r3 i5;
        jnef r1 r2 r3 i5;
        jal r1 r2 i5;
        ret r1;
        retd r1 r2;
        rvrt r1;
    };
}
