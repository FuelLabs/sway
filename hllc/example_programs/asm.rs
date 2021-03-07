contract {
    fn test() -> u32 {
        let a = 5 + 2;
        asm(r1: a, r2) {
            addi r2 r1 i3;
            r2
        }
    }
}
