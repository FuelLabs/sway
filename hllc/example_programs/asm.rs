contract {
    trait MyTrait {
        fn a(b: u32) -> u32;
    } {
        fn b(a: u32) -> u32 { a }
    }

    fn test() -> u32 {
        let a = 5;
        let b: u64 = asm(r1: a, r2) {
            addi r2 r1 i3;
            r2
        };

        b + 1   
        
    }
}
