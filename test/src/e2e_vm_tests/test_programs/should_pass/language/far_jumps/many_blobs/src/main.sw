script;

fn main() -> u64 {
    asm() {
        blob i50000;
    }
    if t() {
        let mut i = 0;
        let mut res = 0;
        while i < 4 {
            if is_even(i) {
                asm() {
                    blob i910000;
                }
                res += 111;
            } else {
                // This one.
                asm() {
                    blob i900000;
                }
                res += 222;
            }
            i += 1;
            if i == 1 {
               continue
            }
        }
        res
    } else {
        if f() {
            asm() {
                blob i520000;
            }
            333
        } else {
            asm() {
                blob i530000;
            }
            444
        }
    }
}

fn f() -> bool {
    asm() {
        zero: bool
    }
}

fn t() -> bool {
    asm() {
        one: bool
    }
}

fn is_even(n: u64) -> bool {
   n % 2 == 0
}
