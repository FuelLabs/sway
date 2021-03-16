library ops {
    trait Add {
        fn add(self, other: Self) -> Self;
    }

    // there should be an error here for excess type params
    impl Add for u64 {
         fn add(self, other: Self) -> Self {
            asm(r1: self, r2: other, r3) {
                add r3 r2 r1 i10;
                r3
            }
         }
    }

    struct Test {
        a: u64,
        b: u64
    }

    fn test() {
        // now, need to work out methods and using the self type on them
        let test = Test { a: 5, b: 5 };
        let y: u64 = test.a;

        let z = y.add(test.b);
    }
}
/*
// the compiler will rename these to the ops, + - / * etc
fn add_u64(a: u64, b: u64) -> u64 {
    asm(r1: a, r2: b, r3) {
        add r3 r2 r1;
        r3
    }
}
fn add_u32(a: u32, b: u32) -> u64 {
    asm(r1: a, r2: b, r3) {
        add r3 r2 r1;
        r3
    }
}
fn add_u16(a: u16, b: u16) -> u64 {
    asm(r1: a, r2: b, r3) {
        add r3 r2 r1;
        r3
    }
}
fn add_u8(a: u8, b: u8) -> u64 {
    asm(r1: a, r2: b, r3) {
        add r3 r2 r1;
        r3
    }
}
*/
