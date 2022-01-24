script;
fn main() -> u64 {
    let a = 255;

    enum X {
        Y: bool,
    }

    impl DoesNotExist for X {
        fn eq(self, other: Self) -> bool {
            asm(r1: self, r2: other, r3) {
                eq r3 r2 r1;
                r3: bool
            }
        }
        fn lt(self, other: Self) -> bool {
            asm(r1: self, r2: other, r3) {
                lt r3 r2 r1;
                r3: bool
            }
        }
        fn gt(self, other: Self) -> bool {
            asm(r1: self, r2: other, r3) {
                gt r3 r2 r1;
                r3: bool
            }
        }
    }
    does_not_exist(4u32);
    if X::Y(true) == X::Y(true) {
        a
    } else {
        a
    }
}
